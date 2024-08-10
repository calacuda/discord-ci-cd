use ci_cd::{Backend, BackendState, CiCdCmd, PipelineName, Repo, RepoName};
use crossbeam::channel::{Receiver, Sender};
use poise::serenity_prelude::futures::lock::Mutex;
use std::{collections::HashMap, str::FromStr, sync::Arc};
use url::Url;

pub mod ci_cd;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Arc<Mutex<Data>>, Error>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, poise::ChoiceParameter, Debug)]
pub enum ShowArgs {
    // #[serde(rename = "pipelines")]
    Pipelines,
    // #[serde(rename = "projects")]
    Projects,
    // #[serde(rename = "repos")]
    Repos,
}

impl FromStr for ShowArgs {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "pipelines" => ShowArgs::Pipelines,
            "projects" => ShowArgs::Projects,
            "repos" => ShowArgs::Repos,
            thing => {
                return Err(format!(
                    "{thing} is not a known entity and thus cannot be shown."
                ))
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct Data {
    pub git_links: HashMap<RepoName, Url>,
    pub backend: Arc<Mutex<Backend>>,
    pub send_cmd: Sender<CiCdCmd>,
    pub get_output: Receiver<String>,
}

/// registers a git repo to be able to CICD it.
#[poise::command(slash_command, prefix_command)]
pub async fn resgister(
    ctx: Context<'_>,
    #[description = "Git Clone link"] git_url: Url,
) -> Result<(), Error> {
    // TODO: add admin check
    match ctx {
        Context::Prefix(data) => {
            let response = if git_url.to_string().ends_with(".git") {
                let repo_name = git_url.path().replacen("/", "", 1).replace(".git", "");
                data.data.lock().await.git_links.insert(repo_name, git_url);
                "added. now tracking the requested repo."
            } else {
                "that is not a valiud git link"
            };

            ctx.say(response).await?;
        }
        Context::Application(data) => {
            let response = if git_url.to_string().ends_with(".git") {
                let repo_name = git_url.path().replacen("/", "", 1).replace(".git", "");
                data.data.lock().await.git_links.insert(repo_name, git_url);
                "added. now tracking the requested repo."
            } else {
                "that is not a valiud git link"
            };

            ctx.say(response).await?;
        }
    }

    Ok(())
}

/// shows state information.
#[poise::command(slash_command, prefix_command)]
pub async fn show(
    ctx: Context<'_>,
    #[description = "Show what? Show this."] showable: ShowArgs,
) -> Result<(), Error> {
    // println!("show {showable:?}");
    // TODO: add admin check

    let data = match ctx {
        Context::Prefix(data) => data.data.lock().await,
        Context::Application(data) => data.data.lock().await,
    };

    let response = match showable {
        ShowArgs::Repos => {
            let mut repos: Vec<String> = data.git_links.clone().into_keys().collect();
            repos.sort();

            format!("{:?}", repos)
        }
        ShowArgs::Projects => "not yet programed".into(),
        ShowArgs::Pipelines => "not yet programmed".into(),
    };

    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn load(
    ctx: Context<'_>,
    #[description = "load this repo"] repo: String,
) -> Result<(), Error> {
    // TODO: add admin check

    // println!("getting data");
    let data = match ctx {
        Context::Prefix(data) => data.data.lock().await,
        Context::Application(data) => data.data.lock().await,
    };
    // println!("got data");

    // let backend_state = data.state.clone();

    let mut backend = data.backend.lock().await;
    let backend_state = backend.state.clone();

    // println!("1");

    let mut set_loaded_repo = || {
        if let Some(repo_url) = data.git_links.get(&repo) {
            backend.state = BackendState::Available {
                repo: Repo {
                    repo_name: repo.clone(),
                    url: repo_url.to_owned(),
                },
            };
            if let Err(e) = data.send_cmd.send(CiCdCmd::Clone(repo_url.clone())) {
                eprintln!("failed to send clone command: {e}");
                format!("failed to clone repo. {e}")
            } else {
                eprintln!("loaded repo");
                format!("loaded {repo}.")
            }
        } else {
            format!("unknown git repo {repo}. try: `/show Repos`")
        }
    };

    // println!("2");

    let response = match backend_state {
        BackendState::NotConfigured => set_loaded_repo(),
        BackendState::Available {
            repo: Repo { repo_name, url: _url },
        } => {
            if repo_name != repo {
                set_loaded_repo()
            } else {
                format!("{repo} is already loaded")
            }
        }
        BackendState::RunningPipeline { 
            repo: Repo { repo_name, url: _url },
            pipeline 
        } => format!("already running {pipeline} from the repository, {repo_name}. a new repo cant be loaded until the current pipeline finishes.")
    };

    // println!("{response}");

    ctx.say(response).await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn run(
    ctx: Context<'_>,
    #[description = "which pipeline to run"] pipeline: PipelineName,
) -> Result<(), Error> {
    // TODO: add admin check

    let data = match ctx {
        Context::Prefix(data) => data.data.lock().await,
        Context::Application(data) => data.data.lock().await,
    };

    let backend = data.backend.lock().await;
    let backend_state = backend.state.clone();

    let response = match backend_state {
        BackendState::NotConfigured => "must `/load` a repo before running a pipeline.".into(),
        BackendState::Available {
            repo: Repo { repo_name: _, url },
        } => {
            data.send_cmd.send(CiCdCmd::Clone(url))?;
            data.send_cmd.send(CiCdCmd::RunPipeline(pipeline.clone()))?;

            format!("started pipline {pipeline}. use `/logs` to get logs.")
        }
        BackendState::RunningPipeline { 
            repo: Repo { repo_name, url: _url },
            pipeline 
        } => format!("already running {pipeline} from the repository, {repo_name}. a new pipline cant be run until the current pipeline finishes.")
    };

    ctx.say(response).await?;

    Ok(())
}

