use crate::addr::AddrResult;
use crate::addr::AddrType;
use crate::predule::*;
use crate::types::LocalUpdate;
use crate::update::UpdateOptions;
use derive_getters::Getters;
use serde_derive::{Deserialize, Serialize};

use orion_error::ErrorOwe;
use std::path::Path;
#[derive(Getters, Clone, Debug, Deserialize, Serialize)]
pub struct Artifact {
    name: String,
    version: String,
    #[serde(alias = "addr")]
    origin_addr: AddrType,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_addr: Option<AddrType>,
    #[serde(default = "default_cache_enable")]
    cache_enable : bool,
    local: String,
}
fn default_cache_enable() -> bool{
    false
}

impl Artifact {
    pub fn new<S: Into<String>, A: Into<AddrType>>(name: S, version: S ,addr: A, local: S) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            origin_addr: addr.into(),
            cache_addr: None,
            cache_enable: false,
            local: local.into(),
        }
    }

    // 直接从远程仓库下载
    pub async fn deploy_repo_to_local(
        &self,
        dest_path: &Path,
        options: &UpdateOptions,
    ) -> AddrResult<UpdateUnit> {
        std::fs::create_dir_all(dest_path).owe_res()?;
        let result = self
            .origin_addr
            .update_local_rename(dest_path, &self.name, options)
            .await?;
        Ok(result)
    }

}

#[derive(Getters, Clone, Debug, Deserialize, Serialize)]
pub struct DockImage {
    cep: String,
    addr: AddrType,
}

#[derive(Getters, Clone, Debug, Deserialize, Serialize)]
pub struct BinPackage {
    cep: String,
    addr: AddrType,
}

#[cfg(test)]
mod tests {

    use home::home_dir;

    use crate::addr::{GitAddr, HttpAddr, };

    use super::*;

    #[ignore = "not run in ci"]
    #[tokio::test]
    async fn test_http_artifact_v1() -> AddrResult<()> {
        let artifact = Artifact::new(
            "hello-word",
            "0.1.0",
            HttpAddr::from("https://github.com/galaxy-sec/hello-word.git"),
            "hello-word",
        );
        let path = home_dir()
            .unwrap_or("UNKOWN".into())
            .join(".cache")
            .join("v1");
        artifact
            .deploy_repo_to_local(&path, &UpdateOptions::default())
            .await?;

        assert!(path.join("hello-word").exists());
        Ok(())
    }

    #[ignore = "not run in ci"]
    #[tokio::test]
    async fn test_http_artifact_v2() -> AddrResult<()> {

        let cache_addr = AddrType::Http(HttpAddr::from(
            "https://dy-sec-generic.pkg.coding.net/galaxy-open/generic/galaxy-init.sh?version=latest",
        ));
        let deploy_type = AddrType::Git(
            GitAddr::from("git@github.com:galaxy-sec/spec_test.git").with_branch("main"),
        );
        let _artifact = Artifact {
            name: "galaxy-init".to_string(),
            version: "0.1.0".to_string(),
            origin_addr: deploy_type,
            cache_addr: Some(cache_addr),
            cache_enable: false,
            local: "galaxy-init".to_string(),
        };
        Ok(())
    }
}
