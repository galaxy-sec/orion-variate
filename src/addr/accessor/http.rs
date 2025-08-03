use crate::{
    addr::{
        AddrReason, AddrResult, AddrType, GitAddr, HttpAddr,
        proxy::create_http_client,
        redirect::{DirectPath, serv::DirectServ},
    },
    predule::*,
    types::RemoteUpdate,
    update::UpdateOptions,
    vars::EnvDict,
};

use getset::{Getters, WithSetters};
use orion_error::{ToStructError, UvsResFrom};
use tokio::io::AsyncWriteExt;
use tracing::info;
use url::Url;

use crate::{types::LocalUpdate, vars::EnvEvalable};

#[derive(Getters, Clone, Debug, WithSetters, Default)]
#[getset(get = "pub")]
pub struct HttpAccessor {
    #[getset(set_with = "pub")]
    redirect: Option<DirectServ>,
}

impl HttpAccessor {
    pub fn get_filename(&self) -> Option<String> {
        let url_str = self
            .redirect
            .as_ref()
            .map(|x| x.redirect(self.url.as_str()).path().to_string())
            .unwrap_or(self.url.clone());
        let url = Url::parse(url_str.as_str()).ok()?;
        url.path_segments()?.next_back().and_then(|s| {
            if s.is_empty() {
                None
            } else {
                Some(s.to_string())
            }
        })
    }
}

impl HttpAccessor {
    pub async fn upload<P: AsRef<Path>>(
        &self,
        addr: &HttpAddr,
        file_path: P,
        method: &str,
    ) -> AddrResult<()> {
        use indicatif::{ProgressBar, ProgressStyle};
        let mut ctx = WithContext::want("upload url");

        let client = create_http_client();
        let file_name = self
            .get_filename()
            .unwrap_or_else(|| "file.bin".to_string());
        ctx.with_path("local file", file_path.as_ref());

        println!(
            "upload : {} => \n {}",
            file_path.as_ref().display(),
            addr.url(),
        );
        let file_content = std::fs::read(file_path).owe_data().with(&ctx)?;
        // 记录本地文件大小
        println!("本地文件大小: {} 字节", file_content.len());
        // 创建进度条
        let content_len = file_content.len() as u64;
        let pb = ProgressBar::new(content_len);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})").owe_logic()?
            .progress_chars("#>-"));

        ctx.with("url", addr.url());
        let mut request = match method.to_uppercase().as_str() {
            "POST" => {
                let part = reqwest::multipart::Part::stream_with_length(file_content, content_len)
                    .file_name(file_name);
                let form = reqwest::multipart::Form::new().part("file", part);
                client.post(addr.url()).multipart(form)
            }
            "PUT" => {
                // PUT方法直接使用文件内容作为请求体，避免multipart额外头部
                client.put(addr.url()).body(file_content)
            }
            _ => {
                return Err(StructError::from_res(format!(
                    "Unsupported HTTP method: {method}",
                )));
            }
        };

        if let (Some(u), Some(p)) = (addr.username(), addr.password()) {
            request = request.basic_auth(u, Some(p));
        }

        let response = request.send().await.owe_res().with(&ctx)?;
        response.error_for_status().owe_res().with(&ctx)?;
        pb.finish_with_message("上传完成");
        Ok(())
    }

    pub async fn download(
        &self,
        addr: &HttpAddr,
        dest_path: &Path,
        options: &UpdateOptions,
    ) -> AddrResult<PathBuf> {
        use indicatif::{ProgressBar, ProgressStyle};

        if dest_path.exists() && options.reuse_cache() {
            info!(target :"spec/addr", "{} exists , ignore!! ",dest_path.display());
            return Ok(dest_path.to_path_buf());
        }
        if dest_path.exists() {
            std::fs::remove_file(dest_path).owe_res()?;
        }
        let mut ctx = WithContext::want("download url");
        ctx.with("url", addr.url());
        //let client = reqwest::Client::new();
        let client = create_http_client();
        let request = if let Some(director) = &self.redirect {
            let proxy_path = director.redirect(addr.url());
            let mut request = client.get(proxy_path.path());
            println!("request url:{}", proxy_path.path());
            match proxy_path {
                DirectPath::Origin(_) => {
                    if let (Some(u), Some(p)) = (addr.username(), addr.password()) {
                        request = request.basic_auth(u, Some(p));
                    }
                }
                DirectPath::Proxy(_, auth_opt) => {
                    if let Some(auth) = auth_opt {
                        request = request.basic_auth(auth.username(), Some(auth.password()));
                    }
                }
            }
            request
        } else {
            let mut request = client.get(addr.url());
            if let (Some(u), Some(p)) = (addr.username(), addr.password()) {
                request = request.basic_auth(u, Some(p));
            }
            request
        };
        //let mut request = client.get(&self.url);

        let mut response = request.send().await.owe_res().with(&ctx)?;

        if !response.status().is_success() {
            return Err(StructError::from_res(format!(
                "HTTP request failed: {}",
                response.status()
            )))
            .with(&ctx);
        }

        let total_size = response.content_length().unwrap_or(0);

        ctx.with_path("local", dest_path);
        let mut file = tokio::fs::File::create(&dest_path)
            .await
            .owe_conf()
            .with(&ctx)?;

        // 创建进度条
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})").owe_logic()?
            .progress_chars("#>-"));

        let mut downloaded: u64 = 0;

        while let Some(chunk) = response.chunk().await.owe_data().with(&ctx)? {
            file.write_all(&chunk).await.owe_sys().with(&ctx)?;

            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message("下载完成");
        Ok(dest_path.to_path_buf())
    }
}

#[async_trait]
impl LocalUpdate for HttpAccessor {
    async fn update_local(
        &self,
        addr: &AddrType,
        dest_dir: &Path,
        options: &UpdateOptions,
    ) -> AddrResult<UpdateUnit> {
        let file = self.get_filename();
        let dest_path = dest_dir.join(file.unwrap_or("file.tmp".into()));
        match addr {
            AddrType::Http(http) => Ok(UpdateUnit::from(
                self.download(http, &dest_path, options).await?,
            )),
            _ => Err(AddrReason::Brief(format!("addr type error {addr}")).to_err()),
        }
    }
}

#[async_trait]
impl RemoteUpdate for HttpAccessor {
    async fn update_remote(
        &self,
        addr: &AddrType,
        path: &Path,
        _: &UpdateOptions,
    ) -> AddrResult<UpdateUnit> {
        if !path.exists() {
            return Err(StructError::from_res("path not exist".into()));
        }
        match addr {
            AddrType::Http(http) => {
                self.upload(http, path, "POST").await?;
                if path.is_file() {
                    std::fs::remove_file(path).owe_res()?;
                } else {
                    std::fs::remove_dir_all(path).owe_res()?;
                }
                Ok(UpdateUnit::from(path.to_path_buf()))
            }
            _ => Err(AddrReason::Brief(format!("addr type error {addr}")).to_err()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        addr::{
            AddrResult,
            redirect::{Auth, Rule},
        },
        update::UpdateOptions,
    };

    use super::*;
    use httpmock::{Method::GET, MockServer};

    #[tokio::test(flavor = "current_thread")]
    async fn test_http_auth_download() -> AddrResult<()> {
        // 1. 配置模拟服务器
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/wpflow.txt")
                .header("Authorization", "Basic Z2VuZXJpYy0xNzQ3NTM1OTc3NjMyOjViMmM5ZTliN2YxMTFhZjUyZjAzNzVjMWZkOWQzNWNkNGQwZGFiYzM=");
    then.status(200)
        .header("content-type", "text/html; charset=UTF-8")
        .body("download success");
        });

        // 2. 执行下载
        let temp_dir = PathBuf::from("./test/temp");
        let test_file = temp_dir.join("wpflow.txt");
        if test_file.exists() {
            std::fs::remove_file(&test_file).owe_res()?;
        }
        let http_addr = HttpAddr::from(server.url("/wpflow.txt")).with_credentials(
            "generic-1747535977632",
            "5b2c9e9b7f111af52f0375c1fd9d35cd4d0dabc3",
        );

        let http_accessor = HttpAccessor::default();
        http_accessor
            .update_local(
                &AddrType::from(http_addr),
                &temp_dir,
                &UpdateOptions::for_test(),
            )
            .await?;

        // 3. 验证结果
        assert!(test_file.exists());
        mock.assert();
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_http_auth_download_with_redirect() -> AddrResult<()> {
        // 1. 配置模拟服务器
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/success.txt")
                .header("Authorization", "Basic Z2VuZXJpYy0xNzQ3NTM1OTc3NjMyOjViMmM5ZTliN2YxMTFhZjUyZjAzNzVjMWZkOWQzNWNkNGQwZGFiYzM=");
    then.status(200)
        .header("content-type", "text/html; charset=UTF-8")
        .body("download success");
        });

        // 2. 执行下载
        let temp_dir = PathBuf::from("./test/temp");
        let test_file = temp_dir.join("success.txt");
        if test_file.exists() {
            std::fs::remove_file(&test_file).owe_res()?;
        }
        let redirect = DirectServ::from_rule(
            Rule::new(server.url("/unkonw*"), server.url("/success")),
            Some(Auth::new(
                "generic-1747535977632",
                "5b2c9e9b7f111af52f0375c1fd9d35cd4d0dabc3",
            )),
        );
        let http_addr = HttpAddr::from(server.url("/unkonw.txt"));
        let http_accessor = HttpAccessor::default().with_redirect(Some(redirect));

        http_accessor
            .update_local(
                &AddrType::from(http_addr),
                &temp_dir,
                &UpdateOptions::for_test(),
            )
            .await?;

        // 3. 验证结果
        assert!(test_file.exists());
        mock.assert();
        Ok(())
    }
    #[ignore = "need more time"]
    #[tokio::test(flavor = "current_thread")]
    async fn test_http_addr() -> AddrResult<()> {
        let path = PathBuf::from("/tmp");
        let addr = HttpAddr::from("https://dy-sec-generic.pkg.coding.net/sec-hub/generic/warp-flow/wpflow?version=1.0.89-alpha")
            .with_credentials(
                "generic-1747535977632",
                "5b2c9e9b7f111af52f0375c1fd9d35cd4d0dabc3",
            );
        let http_accessor = HttpAccessor::default();
        http_accessor
            .update_local(&AddrType::from(addr), &path, &UpdateOptions::for_test())
            .await?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_http_upload_post() -> AddrResult<()> {
        // 1. 配置模拟服务器
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/upload")
                .header_exists("content-type")  // 检查 multipart 头
                .header("Authorization", "Basic Z2VuZXJpYy0xNzQ3NTM1OTc3NjMyOjViMmM5ZTliN2YxMTFhZjUyZjAzNzVjMWZkOWQzNWNkNGQwZGFiYzM=");
            then.status(200)
                .body("upload success");
        });

        // 2. 创建临时测试文件
        let temp_dir = tempfile::tempdir().owe_res()?;
        let file_path = temp_dir.path().join("test.txt");
        tokio::fs::write(&file_path, "test content")
            .await
            .owe_sys()?;

        // 3. 执行上传
        let http_addr = HttpAccessor::from(server.url("/upload")).with_credentials(
            "generic-1747535977632",
            "5b2c9e9b7f111af52f0375c1fd9d35cd4d0dabc3",
        );

        http_addr.upload(&file_path, "POST").await?;

        // 4. 验证结果
        mock.assert();
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_http_upload_put() -> AddrResult<()> {
        // 1. 配置模拟服务器
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::PUT)
                .path("/upload_put")
                .header("Authorization", "Basic Z2VuZXJpYy0xNzQ3NTM1OTc3NjMyOjViMmM5ZTliN2YxMTFhZjUyZjAzNzVjMWZkOWQzNWNkNGQwZGFiYzM="); // 移除content-type检查，PUT请求不使用multipart
            then.status(200).body("upload success");
        });

        // 2. 创建临时测试文件
        let temp_dir = tempfile::tempdir().owe_res()?;
        let file_path = temp_dir.path().join("test_put.txt");
        tokio::fs::write(&file_path, "test put content")
            .await
            .owe_sys()?;

        // 3. 执行上传
        let http_addr = HttpAccessor::from(server.url("/upload_put")).with_credentials(
            "generic-1747535977632",
            "5b2c9e9b7f111af52f0375c1fd9d35cd4d0dabc3",
        );

        http_addr.upload(&file_path, "PUT").await?;

        // 4. 验证结果
        mock.assert();
        Ok(())
    }
}

#[cfg(test)]
mod tests2 {
    use super::*;

    #[test]
    fn test_get_filename_with_regular_url() {
        let addr = HttpAccessor::from("http://example.com/file.txt");
        assert_eq!(addr.get_filename(), Some("file.txt".to_string()));
    }

    #[test]
    fn test_get_filename_with_query_params() {
        let addr = HttpAccessor::from("http://example.com/file.txt?version=1.0");
        assert_eq!(addr.get_filename(), Some("file.txt".to_string()));
    }

    #[test]
    fn test_get_filename_with_fragment() {
        let addr = HttpAccessor::from("http://example.com/file.txt#section1");
        assert_eq!(addr.get_filename(), Some("file.txt".to_string()));
    }

    #[test]
    fn test_get_filename_with_multiple_path_segments() {
        let addr = HttpAccessor::from("http://example.com/path/to/file.txt");
        assert_eq!(addr.get_filename(), Some("file.txt".to_string()));
    }

    #[test]
    fn test_get_filename_with_trailing_slash() {
        let addr = HttpAccessor::from("http://example.com/path/");
        assert_eq!(addr.get_filename(), None);
    }

    #[test]
    fn test_get_filename_with_empty_path() {
        let addr = HttpAccessor::from("http://example.com");
        assert_eq!(addr.get_filename(), None);
    }

    #[test]
    fn test_get_filename_with_invalid_url() {
        let addr = HttpAccessor::from("not a valid url");
        assert_eq!(addr.get_filename(), None);
    }

    #[test]
    fn test_get_filename_with_encoded_characters() {
        let addr = HttpAccessor::from("http://example.com/file%20name.txt");
        assert_eq!(addr.get_filename(), Some("file%20name.txt".to_string()));
    }
}

#[cfg(test)]
mod test3 {
    use super::*;
    use httpmock::MockServer;

    #[tokio::test(flavor = "current_thread")]
    async fn test_http_upload_post() -> AddrResult<()> {
        // 1. 配置模拟服务器
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/upload")
                .header_exists("content-type")  // 检查 multipart 头
                .header("Authorization", "Basic Z2VuZXJpYy0xNzQ3NTM1OTc3NjMyOjViMmM5ZTliN2YxMTFhZjUyZjAzNzVjMWZkOWQzNWNkNGQwZGFiYzM=");
            then.status(200)
                .body("upload success");
        });

        // 2. 创建临时测试文件
        let temp_dir = tempfile::tempdir().owe_res()?;
        let file_path = temp_dir.path().join("test.txt");
        tokio::fs::write(&file_path, "test content")
            .await
            .owe_sys()?;

        // 3. 执行上传
        let http_addr = HttpAccessor::from(server.url("/upload")).with_credentials(
            "generic-1747535977632",
            "5b2c9e9b7f111af52f0375c1fd9d35cd4d0dabc3",
        );

        http_addr
            .update_remote(&file_path, &UpdateOptions::for_test())
            .await?;

        // 4. 验证结果
        mock.assert();
        assert!(!file_path.exists());
        Ok(())
    }
}
