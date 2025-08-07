use crate::{
    addr::{
        AddrReason, AddrResult, Address, HttpResource, http::filename_of_url,
        proxy::create_http_client, redirect::serv::RedirectService,
    },
    predule::*,
    types::ResourceDownloader,
    update::{DownloadOptions, HttpMethod, UploadOptions},
};

use getset::{Getters, WithSetters};
use orion_error::{ToStructError, UvsResFrom};
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, instrument};

use crate::types::ResourceUploader;

#[derive(Getters, Clone, Debug, WithSetters, Default)]
#[getset(get = "pub")]
pub struct HttpAccessor {
    #[getset(set_with = "pub")]
    redirect: Option<RedirectService>,
}

impl HttpAccessor {
    #[instrument(
        target = "orion_variate::addr::http",
        skip(self, file_path),
        fields(
            file_path = %file_path.as_ref().display(),
            url = %addr.url(),
            method = ?method,
        ),
    )]
    pub async fn upload<P: AsRef<Path>>(
        &self,
        addr: &HttpResource,
        file_path: P,
        method: &HttpMethod, //method: &str,
    ) -> AddrResult<()> {
        use indicatif::{ProgressBar, ProgressStyle};
        let mut ctx = WithContext::want("upload url");
        let addr = if let Some(direct_serv) = &self.redirect {
            direct_serv.direct_http_addr(addr.clone())
        } else {
            addr.clone()
        };

        let client = create_http_client();
        let file_name = filename_of_url(addr.url()).unwrap_or_else(|| "file.bin".to_string());
        ctx.with_path("local file", file_path.as_ref());

        info!(
            target: "orion_variate::addr::http",
            file_path = %file_path.as_ref().display(),
            url = %addr.url(),
            method = ?method,
            file_name = file_name,
            "upload started"
        );
        let file_content = std::fs::read(file_path).owe_data().with(&ctx)?;
        debug!(
            target: "orion_variate::addr::http",
            file_size = file_content.len(),
            "local file read"
        );
        // 创建进度条
        let content_len = file_content.len() as u64;
        let pb = ProgressBar::new(content_len);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})").owe_logic()?
            .progress_chars("#>-"));

        ctx.with("url", addr.url());
        let mut request = match method {
            HttpMethod::Post => {
                let part = reqwest::multipart::Part::stream_with_length(file_content, content_len)
                    .file_name(file_name);
                let form = reqwest::multipart::Form::new().part("file", part);
                client.post(addr.url()).multipart(form)
            }
            HttpMethod::Put => {
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

        debug!(
            target: "orion_variate::addr::http",
            url = %addr.url(),
            "sending http download request"
        );
        let response = request.send().await.owe_res().with(&ctx)?;
        response.error_for_status().owe_res().with(&ctx)?;
        pb.finish_with_message("上传完成");
        info!("upload completed");
        Ok(())
    }

    #[instrument(
        target = "orion_variate::addr::http",
        skip(self, dest_path),
        fields(
            url = %addr.url(),
            dest_path = %dest_path.display(),
            cache_reuse = options.reuse_cache(),
        ),
    )]
    pub async fn download(
        &self,
        addr: &HttpResource,
        dest_path: &Path,
        options: &DownloadOptions,
    ) -> AddrResult<PathBuf> {
        use indicatif::{ProgressBar, ProgressStyle};
        let addr = if let Some(direct_serv) = &self.redirect {
            direct_serv.direct_http_addr(addr.clone())
        } else {
            addr.clone()
        };

        if dest_path.exists() && options.reuse_cache() {
            info!(
                target: "orion_variate::addr::http",
                path = %dest_path.display(),
                "file already exists, skipping download due to reuse_cache"
            );
            return Ok(dest_path.to_path_buf());
        }
        if dest_path.exists() {
            std::fs::remove_file(dest_path).owe_res()?;
        }
        let mut ctx = WithContext::want("download url");
        ctx.with("url", addr.url());
        //let client = reqwest::Client::new();
        let client = create_http_client();
        let mut request = client.get(addr.url());
        if let (Some(u), Some(p)) = (addr.username(), addr.password()) {
            request = request.basic_auth(u, Some(p));
        }

        println!("downlaod from :{}", addr.url());
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

        debug!(
            target: "orion_variate::addr::http",
            url = %addr.url(),
            total_size = total_size,
            "starting download stream"
        );
        while let Some(chunk) = response.chunk().await.owe_data().with(&ctx)? {
            file.write_all(&chunk).await.owe_sys().with(&ctx)?;

            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message("下载完成");
        debug!(
            target: "orion_variate::addr::http",
            path = %dest_path.display(),
            "download completed"
        );
        Ok(dest_path.to_path_buf())
    }
}

#[async_trait]
impl ResourceDownloader for HttpAccessor {
    #[instrument(
        target = "orion_variate::addr::http",
        skip(self, dest_dir, options),
        fields(
            addr = %addr,
            dest_dir = %dest_dir.display(),
        ),
    )]
    async fn download_to_local(
        &self,
        addr: &Address,
        dest_dir: &Path,
        options: &DownloadOptions,
    ) -> AddrResult<UpdateUnit> {
        match addr {
            Address::Http(http) => {
                let target_path = if dest_dir.is_dir() {
                    let file = filename_of_url(http.url());
                    &dest_dir.join(file.unwrap_or("file.tmp".into()))
                } else {
                    dest_dir
                };
                Ok(UpdateUnit::from(
                    self.download(http, target_path, options).await?,
                ))
            }
            _ => Err(AddrReason::Brief(format!("addr type error {addr}")).to_err()),
        }
    }
}

#[async_trait]
impl ResourceUploader for HttpAccessor {
    #[instrument(
        target = "orion_variate::addr::http",
        skip(self, path, options),
        fields(
            addr = %addr,
            path = %path.display(),
        ),
    )]
    async fn upload_from_local(
        &self,
        addr: &Address,
        path: &Path,
        options: &UploadOptions,
    ) -> AddrResult<UpdateUnit> {
        let _ = options; // Suppress unused variable warning for now
        if !path.exists() {
            return Err(StructError::from_res("path not exist".into()));
        }
        match addr {
            Address::Http(http) => {
                //TODO: use options.http_method
                self.upload(http, path, options.http_method()).await?;
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
            redirect::{AuthConfig, Rule},
        },
        tools::test_init,
        update::DownloadOptions,
    };

    use super::*;
    use httpmock::{Method::GET, MockServer};
    use orion_error::TestAssertWithMsg;
    use orion_infra::path::ensure_path;

    #[tokio::test(flavor = "current_thread")]
    async fn test_http_auth_download_no() -> AddrResult<()> {
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
        let temp_dir = PathBuf::from("./tests/temp");
        let test_file = temp_dir.join("wpflow.txt");
        if test_file.exists() {
            std::fs::remove_file(&test_file).owe_res()?;
        }
        let http_addr = HttpResource::from(server.url("/wpflow.txt")).with_credentials(
            "generic-1747535977632",
            "5b2c9e9b7f111af52f0375c1fd9d35cd4d0dabc3",
        );

        let http_accessor = HttpAccessor::default();
        http_accessor
            .download_to_local(
                &Address::from(http_addr),
                &temp_dir,
                &DownloadOptions::for_test(),
            )
            .await?;

        // 3. 验证结果
        assert!(test_file.exists());
        mock.assert();
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_http_auth_download_with_redirect() -> AddrResult<()> {
        test_init();
        // 1. 配置模拟服务器
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/success.txt")
                .header("Authorization", "Basic Z2VuZXJpYy0xNzQ3NTM1OTc3NjMyOjViMmM5ZTliN2YxMTFhZjUyZjAzNzVjMWZkOWQzNWNkNGQwZGFiYzM=");
            then.status(200)
                .header("content-type", "text/html; charset=UTF-8")
                .body("download success");
        });

        // 2. 执行下载
        let temp_dir = PathBuf::from("./tests/temp");
        ensure_path(&temp_dir).assert("path");
        let test_file = temp_dir.join("unkonw.txt");
        if test_file.exists() {
            std::fs::remove_file(&test_file).owe_res()?;
        }
        let redirect = RedirectService::from_rule(
            Rule::new(server.url("/unkonw*"), server.url("/success")),
            Some(AuthConfig::new(
                "generic-1747535977632",
                "5b2c9e9b7f111af52f0375c1fd9d35cd4d0dabc3",
            )),
        );
        let http_addr = HttpResource::from(server.url("/unkonw.txt"));

        let http_accessor = HttpAccessor::default().with_redirect(Some(redirect));
        http_accessor
            .download_to_local(
                &Address::from(http_addr),
                &temp_dir,
                &DownloadOptions::for_test(),
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
        let addr = HttpResource::from(
            "https://dy-sec-generic.pkg.coding.net/sec-hub/generic/warp-flow/wpflow?version=1.0.89-alpha",
        )
        .with_credentials(
                    "generic-1747535977632",
                    "5b2c9e9b7f111af52f0375c1fd9d35cd4d0dabc3",
                );
        let http_accessor = HttpAccessor::default();
        http_accessor
            .download_to_local(&Address::from(addr), &path, &DownloadOptions::for_test())
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
        let http_addr = HttpResource::from(server.url("/upload")).with_credentials(
            "generic-1747535977632",
            "5b2c9e9b7f111af52f0375c1fd9d35cd4d0dabc3",
        );
        let http_accessor = HttpAccessor::default();

        http_accessor
            .upload(&http_addr, &file_path, &HttpMethod::Post)
            .await?;

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
        let http_addr = HttpResource::from(server.url("/upload_put")).with_credentials(
            "generic-1747535977632",
            "5b2c9e9b7f111af52f0375c1fd9d35cd4d0dabc3",
        );
        let http_accessor = HttpAccessor::default();

        http_accessor
            .upload(&http_addr, &file_path, &HttpMethod::Put)
            .await?;

        // 4. 验证结果
        mock.assert();
        Ok(())
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
        let http_addr = HttpResource::from(server.url("/upload")).with_credentials(
            "generic-1747535977632",
            "5b2c9e9b7f111af52f0375c1fd9d35cd4d0dabc3",
        );
        let http_accessor = HttpAccessor::default();

        http_accessor
            .upload_from_local(
                &Address::from(http_addr),
                &file_path,
                &UploadOptions::new().method(HttpMethod::Post),
            )
            .await?;

        // 4. 验证结果
        mock.assert();
        assert!(!file_path.exists());
        Ok(())
    }
}
