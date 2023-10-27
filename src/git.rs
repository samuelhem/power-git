use std::any::{self, Any};

use gitlab::api::projects;

pub trait Initializable<'a> {
    fn init_repo(&self, name: String) -> Box<dyn Any>;
}

pub struct GitlabClient {
    client: gitlab::Gitlab,
}

impl GitlabClient {
    pub fn new(url: &str, token: &str) -> anyhow::Result<Self> {
        let client = gitlab::Gitlab::new(url, token)?;
        Ok(GitlabClient { client })
    }
}

impl<'a> Initializable<'a> for GitlabClient {
    fn init_repo(&self, name: String) -> Box<dyn Any> {
        Box::new(projects::CreateProject::builder()
            .name(name)
            .visibility(gitlab::api::common::VisibilityLevel::Private)
            .default_branch("main")
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create project: {}", e)))
    }
}
