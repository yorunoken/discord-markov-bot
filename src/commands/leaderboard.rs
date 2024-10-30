use rusqlite::{params, Connection};
use serenity::all::{
    CommandInteraction, CommandOptionType, CreateCommand, CreateCommandOption, CreateEmbed,
};
use serenity::all::{EditInteractionResponse, ResolvedValue};
use std::fmt::Write;

use serenity::prelude::*;
use serenity::Error;

use std::collections::HashMap;

pub async fn execute(ctx: &Context, command: &CommandInteraction) -> Result<(), Error> {
    command.defer(&ctx.http).await?;

    let guild_id = match command.guild_id {
        Some(s) => s,
        _ => return Ok(()),
    };

    let options = &command.data.options();

    let member_id = options
        .iter()
        .find(|opt| opt.name == "user")
        .and_then(|opt| {
            if let ResolvedValue::User(user, _) = &opt.value {
                Some(user.id.get())
            } else {
                None
            }
        });

    let excludes = options
        .iter()
        .find(|opt| opt.name == "exclude_word")
        .and_then(|opt| {
            if let ResolvedValue::String(s) = &opt.value {
                Some(s.to_lowercase())
            } else {
                None
            }
        });

    let excludes_array: Option<Vec<String>> = excludes.map(|v| {
        v.split(",")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .collect()
    });

    let min_word_length = options
        .iter()
        .find(|opt| opt.name == "min_word_length")
        .and_then(|opt| {
            if let ResolvedValue::Integer(i) = &opt.value {
                Some(*i as usize)
            } else {
                None
            }
        })
        .unwrap_or(0);

    let selected_word = options
        .iter()
        .find(|opt| opt.name == "word")
        .and_then(|opt| {
            if let ResolvedValue::String(s) = &opt.value {
                Some(s.to_lowercase())
            } else {
                None
            }
        });

    let limit = 50;

    let prefix_list: Vec<&str> = vec![
        "$", "&", "!", ".", "m.", ">", "<", "[", "]", "@", "#", "%", "^", "*", ",",
    ];

    let embed = tokio::task::spawn_blocking(move || {
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

        let mut query = String::from("SELECT content, author_id FROM messages WHERE guild_id = ?1");

        if member_id.is_some() {
            let _ = write!(query, " AND author_id = ?2");
        }

        let mut stmt = conn.prepare(&query).unwrap();

        let row_mapper = |row: &rusqlite::Row| -> rusqlite::Result<(String, u64)> {
            Ok((row.get(0)?, row.get(1)?))
        };

        let sentences_iter = if let Some(member_id) = member_id {
            stmt.query_map(params![guild_id.get(), member_id], row_mapper)
        } else {
            stmt.query_map(params![guild_id.get()], row_mapper)
        }
        .unwrap();

        let sentences: Vec<(String, u64)> = sentences_iter.map(|result| result.unwrap()).collect();

        let mut word_counts: HashMap<String, HashMap<u64, usize>> = HashMap::new();

        for (content, author_id) in sentences {
            for word in content.split_whitespace() {
                let word = word.to_lowercase();

                if word.len() < min_word_length {
                    continue;
                }

                if let Some(selected_word) = &selected_word {
                    println!("selected: {} | current: {}", selected_word, word);
                    if *selected_word != word {
                        continue;
                    }
                }

                if let Some(excludes) = &excludes_array {
                    if excludes.contains(&word) {
                        continue;
                    }
                }

                if prefix_list.iter().any(|&prefix| word.starts_with(prefix)) {
                    continue;
                }

                let author_counts = word_counts.entry(word).or_insert_with(HashMap::new);
                *author_counts.entry(author_id).or_insert(0) += 1;
            }
        }

        let mut leaderboard: Vec<(String, u64, usize)> = if let Some(selected_word) = selected_word
        {
            word_counts
                .get(&selected_word)
                .map(|author_counts| {
                    author_counts
                        .iter()
                        .map(|(&author_id, &count)| (selected_word.clone(), author_id, count))
                        .collect()
                })
                .unwrap_or_default()
        } else {
            word_counts
                .into_iter()
                .map(|(word, author_counts)| {
                    let (top_author, top_count) = author_counts
                        .into_iter()
                        .max_by_key(|&(_, count)| count)
                        .unwrap();
                    (word, top_author, top_count)
                })
                .collect()
        };

        leaderboard.sort_by_key(|&(_, _, count)| std::cmp::Reverse(count));
        leaderboard.truncate(limit);

        let mut description = String::new();
        const MAX_DESCRIPTION_LENGTH: usize = 4000;

        for (index, (word, author_id, count)) in leaderboard.iter().enumerate() {
            let entry = format!(
                "{index}. **{word}**: {word_count} (by <@{author}>)\n",
                index = index + 1,
                word = word,
                word_count = count,
                author = author_id
            );

            if description.len() + entry.len() > MAX_DESCRIPTION_LENGTH {
                description.push_str("...");
                break;
            }

            description.push_str(&entry);
        }

        description = description.trim_end().to_string();

        EditInteractionResponse::new().embed(
            CreateEmbed::new()
                .title(format!("Word Leaderboard for {}", guild_id))
                .description(description),
        )
    })
    .await
    .unwrap();

    command.edit_response(&ctx.http, embed).await?;
    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("leaderboard")
        .description("Get the leaderboard of a server")
        .add_option(CreateCommandOption::new(
            serenity::all::CommandOptionType::User,
            "user",
            "Get a user's messages",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            "word",
            "Get the leaderboard of a word",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            "exclude_word",
            "Excludes a word, usage: `word,to,exclude`",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::Integer,
            "min_word_length",
            "Minimum word length to fetch from database",
        ))
}
