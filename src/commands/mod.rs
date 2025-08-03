pub mod generate;
pub mod guess;
pub mod leaderboard;
pub mod ping;

use serenity::all::{CommandInteraction, CreateCommand};
use serenity::futures::future::BoxFuture;
use serenity::prelude::*;
use serenity::Error;

type CommandFn = for<'a> fn(
    &'a Context,            // Command context, `ctx`
    &'a CommandInteraction, // Command interaction, `command`
) -> BoxFuture<'a, Result<(), Error>>;

#[derive(Debug)]
pub struct Command {
    pub name: String,
    pub exec: CommandFn,
}

pub fn commands_vecs() -> Vec<Command> {
    vec![
        Command {
            name: "ping".into(),
            exec: |ctx, command| Box::pin(ping::execute(ctx, command)),
        },
        Command {
            name: "guess".into(),
            exec: |ctx, command| Box::pin(guess::execute(ctx, command)),
        },
        Command {
            name: "generate".into(),
            exec: |ctx, command| Box::pin(generate::execute(ctx, command)),
        },
        Command {
            name: "leaderboard".into(),
            exec: |ctx, command| Box::pin(leaderboard::execute(ctx, command)),
        },
    ]
}

pub fn register_vecs() -> Vec<CreateCommand> {
    vec![
        ping::register(),
        generate::register(),
        leaderboard::register(),
        guess::register(),
    ]
}
