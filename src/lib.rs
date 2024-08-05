use anyhow::{ensure, Result};
use clap::{Parser, Subcommand};
use serenity::async_trait;
use serenity::futures::lock::Mutex;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::str::FromStr;
use std::sync::Arc;
use url::Url;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
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

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Register { git_link: Url },
    Show { showable: ShowArgs },
    // Help,
}

#[derive(Default)]
pub struct Data {
    pub git_links: Vec<Url>,
}

#[derive(Default)]
pub struct Handler {
    data: Arc<Mutex<Data>>,
}

impl Handler {
    async fn register(&self, git_link: Url) -> Result<String> {
        ensure!(
            git_link.to_string().ends_with(".git") && git_link.to_file_path().is_ok(),
            "the provided link was not a valid git clone link"
        );

        let project_name = git_link
            .to_file_path()
            .unwrap()
            .to_string_lossy()
            .replace(".git", "");

        self.data.lock().await.git_links.push(git_link.clone());

        Ok(format!(
            "added project `{project_name}` from git link {git_link}"
        ))
    }

    async fn show(&self, showable: ShowArgs) -> Result<String> {
        Ok(match showable {
            ShowArgs::Repos => format!("{:?}", self.data.lock().await.git_links),
            show_thing => format!("/show {show_thing:?} is not implemented yet"),
        })
    }
}

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event. This is called whenever a new message is received.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be
    // dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        println!("{msg:?}");

        if !msg.content.starts_with("/") {
            return;
        } else if msg.content.to_lowercase().starts_with("/help") {
            if let Err(why) = msg
                .channel_id
                .say(&ctx.http, "on a scale of 1 to 10 my firned you're fucked!")
                .await
            {
                println!("Error sending message: {why:?}");
            }

            return;
        }

        // parse command using clap
        let args = Cli::parse_from(msg.content.replacen("/", "", 1).split_whitespace());

        let res = match args.command {
            Commands::Register { git_link } => self.register(git_link).await,
            Commands::Show { showable } => self.show(showable).await,
            // Commands::Help => Ok("on a scale of 1 to 10 my firned you're fucked!".into()),
        };

        println!("{res:?}");

        if let Err(why) = msg.channel_id.say(&ctx.http, format!("{res:?}")).await {
            println!("Error sending message: {why:?}");
        }

        // if msg.content == "!ping" {
        //     // Sending a message can fail, due to a network error, an authentication error, or lack
        //     // of permissions to post in the channel, so log to stdout when some error happens,
        //     // with a description of it.
        //     if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
        //         println!("Error sending message: {why:?}");
        //     }
        // }
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}
