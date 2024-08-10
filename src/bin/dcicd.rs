use crossbeam::channel::unbounded;
use discord_ci_cd::{
    ci_cd::{run_backend, Backend},
    load, resgister, run, show, Data,
};
use poise::serenity_prelude::{self as serenity, futures::lock::Mutex};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::spawn;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    // TODO: read token from config file
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = serenity::GatewayIntents::non_privileged();
    let (cmd_tx, cmd_rx) = unbounded();
    let (log_tx, log_rx) = unbounded();
    let backend = Arc::new(Mutex::new(Backend::new(cmd_rx, log_tx)));
    let data = Data {
        git_links: HashMap::default(),
        backend: backend.clone(),
        send_cmd: cmd_tx,
        get_output: log_rx,
    };

    spawn(run_backend(backend));

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![resgister(), show(), load(), run()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Arc::new(Mutex::new(data)))
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
