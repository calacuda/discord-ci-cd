use actix_web::{post, web, Result};
use discord_ci_cd::ci_cd::{PipelineName, Repo};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use url::Url;

type UserId = u64;
type AuthToken = String;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
enum PipelineState {
    /// a pipeline is running
    RunningPipeline { repo: Repo, pipeline: PipelineName },
    /// available to run pipelines
    Available { repo: Repo },
    /// no repos is "loaded"
    #[default]
    NotConfigured,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
struct UserState {
    state: PipelineName,
    repos: Vec<Repo>,
    ouput_logs: Option<String>,
    artifacts: Option<Vec<PathBuf>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct AppState {
    user_state: HashMap<UserId, UserState>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct RegisterArgs {
    clone_url: Url,
    user_id: UserId,
    auth: AuthToken,
}

/// end point to register a new git repo
#[post("/register")]
async fn register(data: web::Data<AppState>, body: web::Json<RegisterArgs>) -> Result<String> {
    let auth_token = body.auth.clone();

    // TODO: write verify_auth_token and use it to verify an
    // verify_auth_token(auth_token)?;

    let clone_url = body.clone_url.clone();

    Ok(format!("git repo \"{clone_url}\" registered successfully"))
}

#[actix::main]
async fn main() -> Result<()> {
    Ok(())
}
