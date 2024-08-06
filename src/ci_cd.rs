use std::collections::HashMap;

use url::Url;

pub type PipeLineName = String;
pub type RepoName = String;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Repo {
    pub repo_name: RepoName,
    pub url: Url,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BackendState {
    /// a pipeline is running
    RunningPipeline { repo: Repo, pipeline: PipeLineName },
    /// available to run pipelines
    Available(Repo),
    /// no repos is "loaded"
    NotConfigured,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backend {
    /// tells if the back end is running, available, etc
    pub state: BackendState,
    pub repos: HashMap<RepoName, Url>,
}
