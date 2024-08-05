use discord_ci_cd::Handler;
use serenity::{all::GatewayIntents, Client};
use std::env;

#[tokio::main]
async fn main() {
    // TODO: read token from config file
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = 
        // GatewayIntents::GUILD_MESSAGES
        // | GatewayIntents::DIRECT_MESSAGES
        // GatewayIntents::MESSAGE_CONTENT;
        GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::non_privileged();

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler::default())
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
