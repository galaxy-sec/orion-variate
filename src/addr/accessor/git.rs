use crate::addr::proxy::ProxyConfig;
use crate::addr::redirect::serv::RedirectService;
use crate::addr::{AddrReason, AddrResult, Address, GitRepository};
use crate::{predule::*, tools::get_repo_name, types::{ResourceDownloader, ResourceUploader}, update::UpdateOptions};
use async_trait::async_trait;
use fs_extra::dir::CopyOptions;
use getset::{Getters, Setters, WithSetters};
use git2::{
    BranchType, FetchOptions, MergeOptions, PushOptions, RemoteUpdateFlags, Repository, ResetType,
    build::{CheckoutBuilder, RepoBuilder},
};
use home::home_dir;
use log::warn;
use orion_error::{ToStructError, UvsResFrom};
use orion_infra::auto_exit_log;
use orion_infra::path::ensure_path;

///
/// 支持通过SSH和HTTPS协议访问Git仓库
///
/// # Token认证示例
///

#[derive(Clone, Debug, Default, Getters, Setters, WithSetters)]
#[getset(get = "pub", set = "pub")]
pub struct GitAccessor {
    #[getset(set_with = "pub")]
    redirect: Option<RedirectService>,
    #[getset(set_with = "pub")]
    proxy: Option<ProxyConfig>,
}

impl GitAccessor {
    /// 从环境变量自动加载代理配置
    /// 支持 https_proxy, http_proxy, all_proxy 等标准环境变量
    pub fn with_proxy_from_env(mut self) -> Self {
        self.proxy = ProxyConfig::from_standard_env();
        self
    }
    /// 构建远程回调（包含SSH认证和Token认证）
    fn build_remote_callbacks(&self, addr: &GitRepository) -> git2::RemoteCallbacks<'_> {
        let mut callbacks = git2::RemoteCallbacks::new();
        let ssh_key = addr.ssh_key().clone();
        let ssh_passphrase = addr.ssh_passphrase().clone();
        let token = addr.token().clone();
        let username = addr.username().clone();

        callbacks.credentials(move |url, username_from_url, allowed_types| {
            // 检查URL类型，决定使用哪种认证方式
            let is_https = url.starts_with("https://");

            if is_https {
                // HTTPS协议使用Token认证
                if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
                    // 使用已提供的token，如果没有则尝试从.git-credentials读取
                    let final_token = token.clone().or_else(|| {
                        if let Some(credentials) = GitRepository::read_git_credentials() {
                            // 查找匹配的凭证
                            credentials
                                .iter()
                                .find(|(cred_url, _, _)| url.contains(cred_url))
                                .map(|(_, _, token)| token.clone())
                        } else {
                            None
                        }
                    });

                    // 使用已提供的用户名，如果没有则尝试从.git-credentials读取或默认
                    let final_username = username
                        .clone()
                        .or_else(|| {
                            if let Some(credentials) = GitRepository::read_git_credentials() {
                                credentials
                                    .iter()
                                    .find(|(cred_url, _, _)| url.contains(cred_url))
                                    .map(|(_, username, _)| username.clone())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| username.clone().unwrap_or_else(|| "git".to_string()));

                    if let Some(token) = final_token {
                        // 根据不同的Git平台使用不同的Token格式
                        let actual_username = if final_username == "oauth2" {
                            // GitLab使用oauth2作为用户名
                            "oauth2"
                        } else if final_username == "x-token-auth" {
                            // Bitbucket使用x-token-auth作为用户名
                            "x-token-auth"
                        } else {
                            // 默认使用提供的用户名或git
                            &final_username
                        };
                        git2::Cred::userpass_plaintext(actual_username, &token)
                    } else {
                        // 如果没有token，允许git使用默认的credential helper
                        Err(git2::Error::from_str("需要Token认证但未提供token"))
                    }
                } else {
                    Err(git2::Error::from_str("HTTPS协议不支持所需的认证类型"))
                }
            } else {
                // SSH协议使用密钥认证
                if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                    let username = username_from_url.unwrap_or("git");

                    // 尝试获取SSH密钥路径
                    let key_path = if let Some(custom_key) = &ssh_key {
                        // 使用用户指定的密钥
                        PathBuf::from(custom_key)
                    } else {
                        // 自动查找常见默认密钥
                        find_default_ssh_key()
                            .ok_or_else(|| git2::Error::from_str("无法找到默认SSH密钥"))?
                    };

                    git2::Cred::ssh_key(
                        username,
                        None, // 不使用默认公钥路径
                        &key_path,
                        ssh_passphrase.as_deref(), // 传递密码（如果有）
                    )
                } else {
                    Err(git2::Error::from_str("SSH协议不支持所需的认证类型"))
                }
            }
        });
        callbacks
    }

    /// 更新现有仓库
    fn update_repo(&self, addr: &GitRepository, repo: &Repository) -> Result<(), git2::Error> {
        if !self.is_workdir_clean(repo)? {
            return Err(git2::Error::from_str("工作区有未提交的更改"));
        }
        // 1. 获取远程更新
        self.fetch_updates(addr, repo)?;

        // 2. 处理检出目标（这会切换到指定分支）
        self.checkout_target(addr, repo)?;

        // 3. 执行 pull 操作（合并远程变更）
        self.pull_updates(addr, repo)
    }

    /// 执行 pull 操作：合并远程变更
    fn pull_updates(&self, _addr: &GitRepository, repo: &Repository) -> Result<(), git2::Error> {
        // 获取当前分支信息
        let head = repo.head()?;
        let branch_name = match head.shorthand() {
            Some(name) => name,
            None => return Ok(()), // 分离头状态不需要 pull
        };

        // 获取上游分支信息
        let upstream_branch = format!("origin/{branch_name}");
        let upstream_ref = match repo.find_reference(&upstream_branch) {
            Ok(r) => r,
            Err(_) => return Ok(()), // 没有上游分支
        };

        // 获取当前提交和上游提交
        let current_commit = head.peel_to_commit()?;
        let upstream_commit = upstream_ref.peel_to_commit()?;

        // 如果已经在最新状态，无需操作
        if current_commit.id() == upstream_commit.id() {
            return Ok(());
        }

        // 分析合并可能性
        let annotated_commit = repo.find_annotated_commit(upstream_commit.id())?;
        let analysis = repo.merge_analysis(&[&annotated_commit])?;
        //let analysis = repo.merge_analysis(&[&upstream_commit])?;

        if analysis.0.is_up_to_date() {
            // 已经是最新状态
            Ok(())
        } else if analysis.0.is_fast_forward() {
            // 执行快进合并
            self.fast_forward_merge(repo, &upstream_commit)
        } else {
            // 需要手动合并
            self.merge_upstream(repo, &upstream_commit)
        }
    }

    /// 执行快进合并
    fn fast_forward_merge(
        &self,
        repo: &Repository,
        upstream_commit: &git2::Commit,
    ) -> Result<(), git2::Error> {
        // 获取当前分支名称
        let refname = match repo.head()?.name() {
            Some(name) => name.to_string(),
            None => return Err(git2::Error::from_str("无法获取分支名称")),
        };

        // 更新引用到上游提交
        repo.reference(&refname, upstream_commit.id(), true, "Fast-forward")?;

        // 重置工作区到新提交
        repo.reset(upstream_commit.as_object(), ResetType::Hard, None)?;

        Ok(())
    }

    /// 执行非快进合并 (修复类型错误)
    fn merge_upstream(
        &self,
        repo: &Repository,
        upstream_commit: &git2::Commit,
    ) -> Result<(), git2::Error> {
        // 创建带注释的提交 (修复类型不匹配)
        let annotated_commit = repo.find_annotated_commit(upstream_commit.id())?;

        // 执行合并
        repo.merge(&[&annotated_commit], Some(&mut MergeOptions::new()), None)?;

        // 检查合并状态
        if repo.index()?.has_conflicts() {
            return Err(git2::Error::from_str("合并冲突：需要手动解决"));
        }

        // 创建合并提交
        let head_commit = repo.head()?.peel_to_commit()?;
        let mut index = repo.index()?;
        let tree_oid = index.write_tree()?;
        let tree = repo.find_tree(tree_oid)?;

        repo.commit(
            Some("HEAD"),
            &head_commit.author(),
            &head_commit.committer(),
            "合并远程变更",
            &tree,
            &[&head_commit, upstream_commit],
        )?;

        // 清理合并状态
        repo.cleanup_state()?;

        Ok(())
    }

    /// 检查工作区是否干净
    fn is_workdir_clean(&self, repo: &Repository) -> Result<bool, git2::Error> {
        let mut options = git2::StatusOptions::new();
        options.include_untracked(true);
        options.include_ignored(false);

        let statuses = repo.statuses(Some(&mut options))?;
        Ok(statuses.is_empty())
    }

    fn get_local_repo_name(&self, addr: &GitRepository) -> String {
        let mut name = get_repo_name(addr.repo().as_str()).unwrap_or("unknow".into());
        if let Some(postfix) = addr
            .rev()
            .as_ref()
            .or(addr.tag().as_ref())
            .or(addr.branch().as_ref())
        {
            name = format!("{name}_{postfix}");
        }
        name
    }
}

#[async_trait]
impl ResourceDownloader for GitAccessor {
    async fn download_to_local(
        &self,
        addr: &Address,
        path: &Path,
        options: &UpdateOptions,
    ) -> AddrResult<UpdateUnit> {
        let addr = match addr {
            Address::Git(x) => x,
            _ => return AddrReason::Brief(format!("bad format for git {addr}")).err_result(),
        };
        let name = self.get_local_repo_name(addr);
        let cache_local = home_dir()
            .ok_or(StructError::from_res("unget home".into()))?
            .join(".cache/galaxy");
        ensure_path(&cache_local).owe_logic()?;
        let mut git_local = cache_local.join(name.clone());
        let mut ctx = WithContext::want("update repository");

        ctx.with("repo", addr.repo());
        ctx.with_path("path", &git_local);
        let git_local_copy = git_local.clone();
        let mut flag = auto_exit_log!(
            info!(
                target : "addr/git",
                "update {} to {} success!", addr.repo(),git_local_copy.display()
            ),
            error!(
                target : "addr/git",
                "update {} to {} failed", addr.repo(),git_local_copy.display()
            )
        );
        debug!( target : "addr/git", "update options {:?} where :{} ", options, git_local.display() );
        if git_local.exists() && options.clean_git_cache() {
            std::fs::remove_dir_all(&git_local).owe_logic().with(&ctx)?;
            std::fs::create_dir_all(&git_local).owe_logic().with(&ctx)?;
            warn!(
                target : "addr/git",
                "remove cache {} from {} ", addr.repo(),git_local.display()
            )
        } else {
            debug!( target : "addr/git", "git_local:{} , clean : {} ",  git_local.exists(), options.clean_git_cache() );
        }

        match git2::Repository::open(&git_local) {
            Ok(_re) => {
                debug!(target :"spec", " use repo : {}", git_local.display());
                //not need update git ;
                //self.update_repo(&re).owe_data().with(&ctx)?;
            }
            Err(_) => {
                debug!(target :"spec", "clone repo : {}", git_local.display());
                self.clone_repo(addr, &git_local).owe_data().with(&ctx)?;
            }
        }
        let mut real_path = path.to_path_buf();
        if let Some(sub) = addr.path() {
            git_local = git_local.join(sub);
            if let Some(sub_path) = PathBuf::from(sub).iter().next_back() {
                real_path = real_path.join(sub_path);
            }
        } else {
            real_path = real_path.join(name);
        }
        if real_path.exists() {
            std::fs::remove_dir_all(&real_path).owe_res().with(&ctx)?;
        }

        std::fs::create_dir_all(&real_path).owe_res().with(&ctx)?;
        let options = CopyOptions::new();
        debug!(target:"spec", "src-path:{}", git_local.display() );
        debug!(target:"spec", "dst-path:{}", path.display() );
        ctx.with_path("src-path", &git_local);
        ctx.with_path("dst-path", &real_path);
        fs_extra::copy_items(&[&git_local], path, &options)
            .owe_res()
            .with(&ctx)?;
        flag.mark_suc();
        Ok(UpdateUnit::from(real_path))
    }
}

#[async_trait]
impl ResourceUploader for GitAccessor {
    async fn upload_from_local(
        &self,
        addr: &Address,
        path: &Path,
        options: &UpdateOptions,
    ) -> AddrResult<UpdateUnit> {
        let ctx = WithContext::want("upload to repository");
        if !path.exists() {
            return Err(StructError::from_res("path not exist".into()));
        }
        let temp_path = home_dir().unwrap_or(PathBuf::from("~/")).join(".temp");
        ensure_path(&temp_path).owe_logic()?;

        let target_repo = self.download_to_local(addr, &temp_path, options).await?;

        let addr = match addr {
            Address::Git(x) => x,
            _ => return AddrReason::Brief(format!("bad format for git {addr}")).err_result(),
        };
        // 仓库地址
        let target_repo_in_local_path = &target_repo.position;
        if path.is_file() {
            let filename = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("UNKONW");
            std::fs::copy(path, target_repo_in_local_path.join(filename)).owe_res()?;
            std::fs::remove_file(path).owe_res()?;
        } else {
            let copy_options = CopyOptions::new().overwrite(true).copy_inside(true);
            fs_extra::copy_items(&[path], target_repo_in_local_path, &copy_options).owe_res()?;
            std::fs::remove_dir_all(path).owe_res()?;
        }
        match Repository::open(target_repo_in_local_path) {
            Ok(repo) => {
                let branch = addr.branch().clone();
                self.submit(addr, &repo, branch.unwrap_or("master".into()).as_str())
                    .owe_data()
                    .with(&ctx)?;
            }
            Err(e) => {
                debug!(target :"spec", "Open Local repo : {} is failed! error: {}", addr.repo(), e)
            }
        }
        let name = self.get_local_repo_name(addr);
        std::fs::remove_dir_all(temp_path.join(name)).owe_res()?;
        Ok(UpdateUnit::from(path.to_path_buf()))
    }
}

impl GitAccessor {
    pub fn sync_repo(&self, addr: &GitRepository, target_dir: &Path) -> Result<(), git2::Error> {
        // 尝试打开现有仓库
        match Repository::open(target_dir) {
            Ok(repo) => self.update_repo(addr, &repo),
            Err(_) => self.clone_repo(addr, target_dir),
        }
    }

    /// 克隆新仓库
    fn clone_repo(&self, addr: &GitRepository, target_dir: &Path) -> Result<(), git2::Error> {
        let repo_addr = if let Some(director) = &self.redirect {
            director.direct_git_addr(addr.clone())
        } else {
            addr.clone()
        };
        // 准备回调以支持认证
        //
        let callbacks = self.build_remote_callbacks(&repo_addr); // 使用构建的回调

        let mut flag = auto_exit_log!(
            info!(
                target : "addr/git",
                "clone {} to {} success!",  repo_addr.repo().as_str()  , target_dir.display()         ),
            error!(
                target : "addr/git",
                "clone {} to {} failed", repo_addr.repo().as_str()  , target_dir.display()
            )
        );
        // 配置获取选项
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // 配置代理选项
        if let Some(proxy_config) = &self.proxy {
            let mut proxy_options = git2::ProxyOptions::new();
            proxy_options.url(proxy_config.url().as_str());
            fetch_options.proxy_options(proxy_options);
        }

        // 准备克隆选项
        let mut builder = RepoBuilder::new();
        builder.fetch_options(fetch_options);

        // 执行克隆
        let repo = builder.clone(repo_addr.repo(), target_dir)?;

        flag.mark_suc();
        // 处理检出目标
        self.checkout_target(&repo_addr, &repo)
    }

    /// 获取远程更新
    fn fetch_updates(&self, addr: &GitRepository, repo: &Repository) -> Result<(), git2::Error> {
        // 查找 origin 远程
        let mut remote = repo.find_remote("origin")?;

        // 准备认证回调
        let callbacks = self.build_remote_callbacks(addr); // 使用构建的回调
        // 配置获取选项
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // 配置代理选项
        if let Some(proxy_config) = &self.proxy {
            let mut proxy_options = git2::ProxyOptions::new();
            proxy_options.url(proxy_config.url().as_str());
            fetch_options.proxy_options(proxy_options);
        }

        // 执行获取操作
        remote.fetch(&[] as &[&str], Some(&mut fetch_options), None)?;

        // 更新远程引用
        remote.update_tips(
            None,
            RemoteUpdateFlags::UPDATE_FETCHHEAD,
            git2::AutotagOption::All,
            None,
        )?;

        Ok(())
    }

    /// 处理检出目标（按优先级：rev > tag > branch）
    fn checkout_target(&self, addr: &GitRepository, repo: &Repository) -> Result<(), git2::Error> {
        if let Some(rev) = addr.rev() {
            self.checkout_revision(addr, repo, rev)
        } else if let Some(tag) = addr.tag() {
            self.checkout_tag(addr, repo, tag)
        } else if let Some(branch) = addr.branch() {
            self.checkout_branch(addr, repo, branch)
        } else {
            // 默认检出默认分支
            let head = repo.head()?;
            let _name = head
                .name()
                .ok_or_else(|| git2::Error::from_str("无法获取 HEAD 名称"))?;
            repo.checkout_head(Some(&mut CheckoutBuilder::new().force()))?;
            Ok(())
        }
    }

    /// 检出指定提交
    fn checkout_revision(
        &self,
        _addr: &GitRepository,
        repo: &Repository,
        rev: &str,
    ) -> Result<(), git2::Error> {
        let obj = repo.revparse_single(rev)?;
        repo.checkout_tree(&obj, Some(&mut CheckoutBuilder::new().force()))?;
        repo.set_head_detached(obj.id())?;
        Ok(())
    }

    /// 检出指定标签
    fn checkout_tag(
        &self,
        _addr: &GitRepository,
        repo: &Repository,
        tag: &str,
    ) -> Result<(), git2::Error> {
        let refname = format!("refs/tags/{tag}");
        let obj = repo.revparse_single(&refname)?;
        repo.checkout_tree(&obj, Some(&mut CheckoutBuilder::new().force()))?;
        repo.set_head_detached(obj.id())?;
        Ok(())
    }

    /// 检出指定分支（包括远程分支）
    fn checkout_branch(
        &self,
        _addr: &GitRepository,
        repo: &Repository,
        branch: &str,
    ) -> Result<(), git2::Error> {
        // 尝试查找本地分支
        if let Ok(b) = repo.find_branch(branch, BranchType::Local) {
            // 切换到本地分支
            let refname = b
                .get()
                .name()
                .ok_or_else(|| git2::Error::from_str("无效的分支名称"))?;
            repo.set_head(refname)?;
            repo.checkout_head(Some(&mut CheckoutBuilder::new().force()))?;
            return Ok(());
        }

        // 尝试查找远程分支
        let remote_branch_name = format!("origin/{branch}");
        if let Ok(b) = repo.find_branch(&remote_branch_name, BranchType::Remote) {
            // 创建本地分支并设置跟踪
            let commit = b.get().peel_to_commit()?;
            let mut new_branch = repo.branch(branch, &commit, false)?;
            new_branch.set_upstream(Some(&format!("origin/{branch}")))?;

            // 切换到新分支
            let refname = format!("refs/heads/{branch}");
            repo.set_head(&refname)?;
            repo.checkout_head(Some(&mut CheckoutBuilder::new().force()))?;
            return Ok(());
        }

        Err(git2::Error::from_str(&format!("分支 '{branch}' 不存在",)))
    }

    /// 提交
    fn submit(&self, addr: &GitRepository, repo: &Repository, branch: &str) -> Result<(), git2::Error> {
        info!("git push origin {}", &branch);
        let branch_path = format!("refs/heads/{branch}",);
        // 拉取远程进行更新
        self.fetch_updates(addr, repo)?;
        // 合并远程更新
        let fetch_head = repo.find_annotated_commit(repo.refname_to_id("FETCH_HEAD")?)?;
        let (analysis, _) = repo.merge_analysis(&[&fetch_head])?;

        if analysis.is_fast_forward() {
            let mut reference = repo.find_reference(&branch_path)?;
            reference.set_target(fetch_head.id(), "Fast-forward")?;
            repo.set_head(&branch_path)?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        } else if analysis.is_normal() {
            // 出现合并冲突问题
            return Err(git2::Error::from_str("Merge conflicts detected"));
        }
        // 检查是否有修改
        let mut status_option = git2::StatusOptions::new();
        status_option.include_untracked(true);
        status_option.include_ignored(false);
        let local_repo_status = repo.statuses(Some(&mut status_option))?;
        if local_repo_status.is_empty() {
            return Ok(());
        }
        let mut origin = repo.find_remote("origin")?;

        // git add 操作
        let mut index = repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        let tree = repo.find_tree(index.write_tree()?)?;
        let head = repo.head()?.resolve()?;
        let parent_commit = repo.find_commit(head.target().unwrap())?;
        let signature = git2::Signature::now("dayu-spec", "dayu-sec-spec@dy-sec.com")?;
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "spec:auto commit",
            &tree,
            &[&parent_commit],
        )?;
        // 设置认证信息
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(self.build_remote_callbacks(addr));

        // 配置代理选项
        if let Some(proxy_config) = &self.proxy {
            let mut proxy_options = git2::ProxyOptions::new();
            proxy_options.url(proxy_config.url().as_str());
            push_options.proxy_options(proxy_options);
        }
        origin.push(
            &[&format!("{branch_path}:{branch_path}",)],
            Some(&mut push_options),
        )?;
        info!("push complete");
        Ok(())
    }
}

fn find_default_ssh_key() -> Option<PathBuf> {
    // 获取用户主目录
    let home = home_dir()?;
    let ssh_dir = home.join(".ssh");

    // 尝试的密钥文件列表（按优先级排序）
    let key_files = [
        "id_ed25519", // 首选ed25519
        "id_rsa",     //  THEN 是RSA
        "id_ecdsa",   // 然后是ECDSA
        "identity",   // 通用名称
    ];

    // 检查每个密钥文件是否存在
    for key_file in &key_files {
        let key_path = ssh_dir.join(key_file);
        if key_path.exists() {
            return Some(key_path);
        }
    }

    None
}
#[cfg(test)]
mod tests {
    use crate::addr::redirect::{AuthConfig, Rule};
    use crate::{addr::AddrResult, tools::test_init};

    use super::*;
    use orion_error::{ErrorOwe, TestAssert};
    use tempfile::tempdir;

    //git@e.coding.net:dy-sec/s-devkit/kubeconfig.git

    #[ignore = "need more time"]
    #[tokio::test]
    async fn test_git_addr_update_local() -> AddrResult<()> {
        // 创建临时目录

        let temp_dir = tempdir().owe_res()?;
        let dest_path = temp_dir.path().to_path_buf();

        // 使用一个小型测试仓库（这里使用 GitHub 上的一个测试仓库）
        let git_addr =
            GitRepository::from("https://github.com/galaxy-sec/hello-word.git").with_branch("master"); // 替换为实际测试分支

        let accessor = GitAccessor::default();
        // 执行克隆
        let cloned_v = accessor
            .download_to_local(
                &Address::from(git_addr),
                &dest_path,
                &UpdateOptions::default(),
            )
            .await?;

        // 验证克隆结果
        assert!(cloned_v.position().exists());
        assert!(cloned_v.position().join(".git").exists());

        // 验证分支/标签是否正确检出
        let repo = git2::Repository::open(cloned_v.position()).owe_res()?;
        let head = repo.head().owe_res()?;
        assert!(head.is_branch() || head.is_tag());

        Ok(())
    }

    #[tokio::test]
    async fn test_git_addr_update_local_sub() -> AddrResult<()> {
        // 创建临时目录
        test_init();
        let dest_path = PathBuf::from("./tests/temp/git");
        //let target_path = dest_path.join("git");
        if dest_path.exists() {
            std::fs::remove_dir_all(&dest_path).assert();
        }
        std::fs::create_dir_all(&dest_path).assert();

        let git_addr = GitRepository::from("https://github.com/galaxy-sec/hello-word.git")
            .with_branch("main")
            .with_path("x86"); // 或使用 .tag("v1.0") 测试标签

        // 执行克隆
        let addr_type = Address::Git(git_addr.clone());
        let accessor = GitAccessor::default();
        let git_up = accessor
            .download_to_local(&addr_type, &dest_path, &UpdateOptions::default())
            .await
            .assert();
        assert_eq!(git_up.position(), &dest_path.join("x86"));
        Ok(())
    }

    #[tokio::test]
    async fn test_git_addr_pull_2() -> AddrResult<()> {
        // 创建临时目录
        test_init();
        let dest_path = PathBuf::from("./tests/temp/git2");
        if dest_path.exists() {
            std::fs::remove_dir_all(&dest_path).assert();
        }
        std::fs::create_dir_all(&dest_path).assert();

        let git_addr =
            GitRepository::from("https://github.com/galaxy-sec/hello-word.git").with_branch("main");
        // 执行克隆
        let accessor = GitAccessor::default();
        let git_up = accessor
            .download_to_local(
                &Address::from(git_addr),
                &dest_path,
                &UpdateOptions::default(),
            )
            .await
            .assert();
        assert_eq!(git_up.position(), &dest_path.join("hello-word.git_main"));
        Ok(())
    }

    #[tokio::test]
    async fn test_git_addr_pull_redirect() -> AddrResult<()> {
        // 创建临时目录
        test_init();
        let dest_path = PathBuf::from("./tests/temp/git3");
        if dest_path.exists() {
            std::fs::remove_dir_all(&dest_path).assert();
        }
        std::fs::create_dir_all(&dest_path).assert();
        let redirect = RedirectService::from_rule(
            Rule::new(
                "https://github.com/galaxy-sec/hello-none*",
                "https://github.com/galaxy-sec/hello-word",
            ),
            Some(AuthConfig::new(
                "generic-1747535977632",
                "5b2c9e9b7f111af52f0375c1fd9d35cd4d0dabc3",
            )),
        );

        let git_addr =
            GitRepository::from("https://github.com/galaxy-sec/hello-none.git").with_branch("main");
        let accessor = GitAccessor::default().with_redirect(Some(redirect));
        // 执行克隆
        //   let accessor = GitAccessor::default();
        let git_up = accessor
            .download_to_local(
                &Address::from(git_addr),
                &dest_path,
                &UpdateOptions::default(),
            )
            .await
            .assert();
        assert_eq!(git_up.position(), &dest_path.join("hello-none.git_main"));
        Ok(())
    }

    #[tokio::test]
    async fn test_checkout_specific_branch() -> AddrResult<()> {
        test_init();
        let dest_path = PathBuf::from("./tests/temp/git_branch_test");
        if dest_path.exists() {
            std::fs::remove_dir_all(&dest_path).unwrap();
        }

        // 测试切换到非默认分支
        let git_addr =
            GitRepository::from("https://github.com/galaxy-sec/hello-word.git").with_branch("develop"); // 替换为实际测试分支

        let addr_type = Address::Git(git_addr.clone());
        let accessor = GitAccessor::default();
        let git_up = accessor
            .download_to_local(&addr_type, &dest_path, &UpdateOptions::default())
            .await?;
        let repo = git2::Repository::open(git_up.position().clone()).assert();
        let head = repo.head().assert();
        assert!(head.shorthand().unwrap_or("").contains("develop"));
        Ok(())
    }

    use crate::types::{ResourceDownloader, ResourceUploader};
use crate::{addr::GitRepository, update::UpdateOptions};

    #[ignore = "no run in ci"]
    #[tokio::test]
    async fn test_dir_upload_to_remote_repo() -> AddrResult<()> {
        let temp_dir = tempdir().assert();
        let dir = temp_dir.path().join("version_1");
        let file = dir.join("test.txt");
        std::fs::create_dir_all(&dir).assert();
        std::fs::write(&file, "spec upload local dir to git repo.").assert();

        let git_addr = GitRepository::from("git@github.com:galaxy-sec/spec_test.git").with_branch("main");

        let addr_type = Address::Git(git_addr.clone());
        let accessor = GitAccessor::default();
        let git_up = accessor
            .upload_from_local(&addr_type, &dir, &UpdateOptions::default())
            .await?;
        println!("{:?}", git_up.position);
        Ok(())
    }

    #[ignore = "no run in ci"]
    #[tokio::test]
    async fn test_file_upload_to_remote_repo() -> AddrResult<()> {
        let temp_dir = tempdir().assert();
        let file = temp_dir.path().join("test.txt");

        std::fs::write(&file, "spec upload local file to git repo.").assert();

        let git_addr = GitRepository::from("git@github.com:galaxy-sec/spec_test.git").with_branch("main");

        let addr_type = Address::Git(git_addr.clone());
        let accessor = GitAccessor::default();
        let git_up = accessor
            .upload_from_local(&addr_type, &file, &UpdateOptions::default())
            .await?;
        println!("{:?}", git_up.position);
        Ok(())
    }

    #[test]
    fn test_git_addr_env_token() {
        // 测试环境变量方法（不实际设置环境变量，仅验证方法存在）
        let addr = GitRepository::from("https://github.com/user/repo.git");

        // 验证方法可以调用（实际效果取决于环境变量是否存在）
        let addr = addr.with_env_token("NON_EXISTENT_VAR");
        assert!(addr.token().is_none()); // 环境变量不存在时返回None
    }

    #[test]
    fn test_git_credentials_parsing() {
        // 测试.git-credentials文件解析功能
        // 由于环境变量限制，我们简化测试，只验证方法存在和基本功能
        let _result = GitRepository::read_git_credentials();
        // 无论是否存在.git-credentials文件，方法都应该成功返回
    }

    #[test]
    fn test_git_addr_with_git_credentials() {
        // 测试GitAddr的with_git_credentials方法
        let addr = GitRepository::from("https://github.com/user/repo.git");

        // 验证方法可以调用（实际效果取决于.git-credentials文件是否存在）
        let _addr = addr.with_git_credentials();
        // 无论是否找到凭证，方法都应该成功返回
    }

    #[ignore = "need cnb.cool access"]
    #[tokio::test]
    async fn test_git_addr_cnb_cool_clone() -> AddrResult<()> {
        // 创建临时目录
        test_init();
        let dest_path = PathBuf::from("./tests/temp/cnb_cool_test");
        if dest_path.exists() {
            std::fs::remove_dir_all(&dest_path).unwrap();
        }
        std::fs::create_dir_all(&dest_path).unwrap();

        // 测试cnb.cool仓库克隆
        let git_addr = GitRepository::from("https://cnb.cool/dy-sec/ops/sys-operators/mac-devkit.git")
            .with_branch("main");

        // 执行克隆
        let addr_type = Address::Git(git_addr.clone());
        let git_up = GitAccessor::default()
            .download_to_local(&addr_type, &dest_path, &UpdateOptions::default())
            .await?;

        // 验证克隆结果
        assert!(git_up.position().exists());
        assert!(git_up.position().join(".git").exists());

        // 验证分支是否正确检出
        let repo = git2::Repository::open(git_up.position()).owe_res()?;
        let head = repo.head().owe_res()?;
        assert!(head.is_branch());

        Ok(())
    }

    #[ignore = "need cnb.cool token access"]
    #[tokio::test]
    async fn test_git_addr_cnb_cool_with_token() -> AddrResult<()> {
        // 创建临时目录
        test_init();
        let dest_path = PathBuf::from("./tests/temp/cnb_cool_token_test");
        if dest_path.exists() {
            std::fs::remove_dir_all(&dest_path).unwrap();
        }
        std::fs::create_dir_all(&dest_path).unwrap();

        // 测试cnb.cool仓库克隆（带token认证）
        let git_addr =
            GitRepository::from("https://cnb.cool/dy-sec/ops/mechanism/gxl-dayu.git").with_branch("main");
        //.with_token("5WXpns1c2bISgpoPA8EdhtIOarC"); // 需要替换为实际token
        //.with_token("your-cnb-token"); // 需要替换为实际token

        // 执行克隆
        let git_up = GitAccessor::default()
            .download_to_local(
                &Address::from(git_addr),
                &dest_path,
                &UpdateOptions::default(),
            )
            .await?;

        // 验证克隆结果
        assert!(git_up.position().exists());
        assert!(git_up.position().join(".git").exists());

        Ok(())
    }
}
