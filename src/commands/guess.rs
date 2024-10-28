use std::time::Duration;

use rusqlite::Connection;

use futures::StreamExt;
use serenity::all::{
    ButtonStyle, CommandInteraction, CreateButton, CreateCommand, CreateEmbed,
    CreateInteractionResponse, CreateMessage, EditInteractionResponse, UserId,
};
use serenity::prelude::*;
use serenity::Error;

pub fn register() -> CreateCommand {
    CreateCommand::new("guess").description("Guess who a random message belongs to.")
}

pub async fn execute(ctx: &Context, command: &CommandInteraction) -> Result<(), Error> {
    let _ = command.defer(&ctx.http).await;

    let game_stop_seconds = 180;
    let embed = CreateEmbed::new()
        .title("Random message guesser")
        .description(format!(
            "Welcome to message guesser! Bot picks a random message, you guess who sent it.\n\
            Use nickname, username, or ID to guess.\n\
            Game stops after {stop_minutes} minutes of no correct guesses.\n\
            Good luck!",
            stop_minutes = game_stop_seconds / 60
        ));

    let message = command
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new()
                .embed(embed)
                .button(
                    CreateButton::new("start")
                        .style(ButtonStyle::Success)
                        .label("Start"),
                )
                .button(
                    CreateButton::new("cancel")
                        .style(ButtonStyle::Danger)
                        .label("Cancel"),
                ),
        )
        .await?;

    let interaction = match message
        .await_component_interaction(&ctx.shard)
        .timeout(Duration::from_secs(60))
        .await
    {
        Some(x) => x,
        None => {
            let embed = CreateEmbed::new()
                .title("Random message guesser")
                .description("Game cancelled (Timed out)");

            command
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .embed(embed)
                        .button(
                            CreateButton::new("start")
                                .style(ButtonStyle::Success)
                                .label("Start")
                                .disabled(true),
                        )
                        .button(
                            CreateButton::new("cancel")
                                .style(ButtonStyle::Danger)
                                .label("Cancel")
                                .disabled(true),
                        ),
                )
                .await?;

            return Ok(());
        }
    };

    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Acknowledge)
        .await?;

    match interaction.data.custom_id.as_str() {
        "start" => {
            start_game(ctx, command).await?;
        }
        "cancel" => {
            let embed = CreateEmbed::new()
                .title("Random message guesser")
                .description("Game cancelled (Manual cancellation)");

            command
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .embed(embed)
                        .button(
                            CreateButton::new("start")
                                .style(ButtonStyle::Success)
                                .label("Start")
                                .disabled(true),
                        )
                        .button(
                            CreateButton::new("cancel")
                                .style(ButtonStyle::Danger)
                                .label("Cancel")
                                .disabled(true),
                        ),
                )
                .await?;
        }
        _ => {}
    };

    Ok(())
}

async fn start_game(ctx: &Context, command: &CommandInteraction) -> Result<(), Error> {
    let embed = CreateEmbed::new()
        .title("Random message guesser")
        .description("Game started!");
    command
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new()
                .embed(embed)
                .button(
                    CreateButton::new("start")
                        .style(ButtonStyle::Success)
                        .label("Start")
                        .disabled(true),
                )
                .button(
                    CreateButton::new("cancel")
                        .style(ButtonStyle::Danger)
                        .label("Cancel")
                        .disabled(true),
                ),
        )
        .await?;

    let mut game = Game::new(ctx, command);
    game.start_game().await?;

    Ok(())
}

struct Game<'a> {
    pub ctx: &'a Context,
    pub command: &'a CommandInteraction,
    pub game_ended: bool,
}

impl<'a> Game<'a> {
    pub fn new(ctx: &'a Context, command: &'a CommandInteraction) -> Self {
        Self {
            ctx,
            command,
            game_ended: false,
        }
    }

    pub async fn start_game(&mut self) -> Result<(), Error> {
        loop {
            if self.game_ended {
                break;
            }

            self.new_sentence().await?;
        }

        Ok(())
    }

    pub async fn new_sentence(&mut self) -> Result<(), Error> {
        let (random_message, random_author) =
            match get_random_message(&self.command.guild_id.unwrap().get()) {
                Some(s) => s,
                None => {
                    self.end_game("No message were caught, aborting game.")
                        .await?;
                    return Ok(());
                }
            };
        let random_author = UserId::new(random_author).to_user(&self.ctx.http).await?;

        let embed = self.create_embed(format!("Guess who said this:\n\n```{}```", random_message));

        let message = self
            .command
            .channel_id
            .send_message(
                &self.ctx.http,
                CreateMessage::new()
                    .embed(embed)
                    .button(
                        CreateButton::new("skip")
                            .style(ButtonStyle::Primary)
                            .label("Skip Message"),
                    )
                    .button(
                        CreateButton::new("end")
                            .style(ButtonStyle::Danger)
                            .label("End Game"),
                    ),
            )
            .await?;

        loop {
            let mut interaction_stream = message
                .await_component_interaction(&self.ctx.shard)
                .stream();
            let mut message_stream = self.command.channel_id.await_reply(&self.ctx).stream();

            tokio::select! {
                interaction = interaction_stream.next() => {
                    match interaction {
                        Some(interaction) => {
                            match interaction.data.custom_id.as_str() {
                                "skip" => {
                                    interaction
                                        .create_response(&self.ctx.http, CreateInteractionResponse::Acknowledge)
                                        .await?;
                                    break;
                                }
                                "end" => {
                                    interaction
                                        .create_response(&self.ctx.http, CreateInteractionResponse::Acknowledge)
                                        .await?;
                                    self.end_game("Game ended by user.").await?;
                                    self.game_ended = true;
                                    return Ok(());
                                }
                                _ => {}
                            }
                        }
                        None => {}
                    }
                }
                message_collector = message_stream.next() => {
                    match message_collector {
                        Some(message) => {
                            let display_name = random_author.display_name();
                            let correct_guesses = vec![random_author.name.as_str(), &display_name];

                            if correct_guesses.iter().any(|&correct_guess| {
                                correct_guess.to_lowercase() == message.content.to_lowercase()
                            }) {
                                self.command
                                    .channel_id
                                    .send_message(
                                        &self.ctx.http,
                                        CreateMessage::new().content(format!(
                                            "Correct, <@{}>! The message was sent by `{}`",
                                            message.author.id.get(),
                                            random_author.name
                                        )),
                                    )
                                    .await?;
                                break;
                            }
                        }
                        None => {
                            self.end_game("Time's up! Nobody guessed correctly.")
                                .await?;
                            self.game_ended = true;
                            return Ok(());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn end_game(&mut self, reason: impl Into<String>) -> Result<(), Error> {
        let embed = self.create_embed(reason);

        self.command
            .channel_id
            .send_message(&self.ctx.http, CreateMessage::new().embed(embed))
            .await?;

        self.game_ended = true;

        Ok(())
    }

    fn create_embed(&self, content: impl Into<String>) -> CreateEmbed {
        CreateEmbed::new()
            .title("Random message guesser")
            .description(content)
    }
}

fn get_random_message(guild_id: &u64) -> Option<(String, u64)> {
    let mut conn: Option<Connection> = None;
    for i in 0..=5 {
        match Connection::open("messages.db") {
            Ok(conn_ok) => conn = Some(conn_ok),
            Err(err) => {
                eprintln!("Errored while opening db: {}, i: {}", err, i);
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        };
    }

    let conn = conn.expect("Failed to establish database connection after multiple attempts.");

    let mut stmt = conn
        .prepare("SELECT content, author_id FROM messages WHERE guild_id = ?1 ORDER BY RANDOM() LIMIT 1;")
        .unwrap();

    match stmt.query_row([guild_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?))
    }) {
        Ok((content, author_id)) => Some((content, author_id)),
        Err(_) => None,
    }
}
