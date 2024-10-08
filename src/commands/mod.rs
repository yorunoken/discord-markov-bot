pub mod generate;
pub mod leaderboard;
pub mod ping;

use serenity::futures::future::BoxFuture;
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::Error;

type CommandFn = for<'a> fn(
    &'a Context,     // Command context, `ctx`
    &'a Message,     // Message variable, `msg`
    Vec<&'a str>,    // Arguments of the command, `args`
    &'a String,      // The command's name, `command_name`
    Option<&'a str>, // The command's name, `command_alias`
) -> BoxFuture<'a, Result<(), Error>>;

#[derive(Debug)]
pub struct Command {
    pub name: String,
    pub aliases: Vec<String>,
    pub exec: CommandFn,
}

pub fn prefix_commads_vecs() -> Vec<Command> {
    vec![
        Command {
            name: String::from("ping"),
            aliases: vec![String::from("p")],
            exec: |ctx, msg, args, command_name, command_alias| {
                Box::pin(ping::execute(ctx, msg, args, command_name, command_alias))
            },
        },
        Command {
            name: String::from("generate"),
            aliases: vec![String::from("g")],
            exec: |ctx, msg, args, command_name, command_alias| {
                Box::pin(generate::execute(
                    ctx,
                    msg,
                    args,
                    command_name,
                    command_alias,
                ))
            },
        },
        Command {
            name: String::from("leaderboard"),
            aliases: vec![String::from("lb")],
            exec: |ctx, msg, args, command_name, command_alias| {
                Box::pin(leaderboard::execute(
                    ctx,
                    msg,
                    args,
                    command_name,
                    command_alias,
                ))
            },
        },
    ]
}
