use discord_ci_cd::{resgister, show, Data};
use poise::serenity_prelude::{self as serenity, futures::lock::Mutex};
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // TODO: read token from config file
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![resgister(), show()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Arc::new(Mutex::new(Data::default())))
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
