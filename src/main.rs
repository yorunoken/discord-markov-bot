use dotenv::dotenv;
use rusqlite::Connection;
use serenity::prelude::*;
use std::env;

mod event_handler;
mod markov_chain;

#[tokio::main]
async fn main() {
    // Load the environment variables
    dotenv().ok();

    // Create the messages table if it doesn't exist
    let conn = Connection::open("messages.db").expect("Unable to open database");
    let sql = "CREATE TABLE IF NOT EXISTS messages (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        message_id INTEGER NOT NULL,
        author_id INTEGER NOT NULL,
        channel_id INTEGER NOT NULL,
        guild_id INTEGER NOT NULL,
        sentence TEXT NOT NULL
        );";
    conn.execute(sql, []).expect("Failed to create table");

    let discord_token =
        env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN to be defined in environment.");

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    // Build the Discord client, and pass in our event handler
    let mut client = Client::builder(discord_token, intents)
        .event_handler(event_handler::Handler {})
        .await
        .expect("Error creating client.");

    // Run the Discord client (runs the ready function)
    if let Err(reason) = client.start().await {
        println!("Error starting client: {:?}", reason);
    }
}
