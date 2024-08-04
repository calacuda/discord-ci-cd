use poise::serenity_prelude::futures::lock::Mutex;
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc};
use url::Url;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Arc<Mutex<Data>>, Error>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ShowArgs {
    #[serde(rename = "pipelines")]
    Pipelines,
    #[serde(rename = "projects")]
    Projects,
    #[serde(rename = "repos")]
    Repos,
}

impl FromStr for ShowArgs {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
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

#[derive(Default)]
pub struct Data {
    pub git_links: Vec<Url>,
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
                data.data.lock().await.git_links.push(git_url);
                "added. now tracking the requested repo."
            } else {
                "that is not a valid git link"
            };

            ctx.say(response).await?;
        }
        _ => {}
    }
    Ok(())
}

/// shows state information.
#[poise::command(slash_command, prefix_command)]
pub async fn show(
    ctx: Context<'_>,
    #[description = "Show what? Show this."] showable: ShowArgs,
) -> Result<(), Error> {
    // TODO: add admin check

    // let data = match ctx {
    //     Context::Prefix(data) => data.data.lock().await,
    //     Context::Application(data) => data.data.lock().await,
    // };

    match ctx {
        Context::Prefix(data) => {
            let response = match showable {
                ShowArgs::Repos => format!("{:?}", data.data.lock().await.git_links),
                ShowArgs::Projects => "not yet programed".into(),
                ShowArgs::Pipelines => "not yet programmed".into(),
            };

            ctx.say(response).await?;
        }
        _ => {}
    }
    Ok(())
}
