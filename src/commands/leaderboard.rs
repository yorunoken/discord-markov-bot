use rusqlite::{params, Connection};
use std::fmt::Write;

use serenity::all::CreateEmbed;
use serenity::all::CreateMessage;
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::Error;

use std::collections::HashMap;

pub async fn execute(
    ctx: &Context,
    msg: &Message,
    args: Vec<&str>,
    _command_name: &String,
    _command_alias: Option<&str>,
) -> Result<(), Error> {
    let guild_id = match msg.guild_id {
        Some(s) => s,
        _ => return Ok(()),
    };

    // Get pairs of `=`
    let mut pairs = HashMap::new();

    for arg in args {
        if let Some((key, value)) = arg.split_once("=") {
            pairs.insert(key, value);
        }
    }

    let excludes_array: Option<Vec<String>> = match pairs.get("excludes") {
        Some(value) => Some(
            value
                .split(",")
                .filter(|s| !s.is_empty())
                .map(|s| s.to_lowercase())
                .collect(),
        ),
        None => None,
    };

    let limit = pairs
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10);

    let min_word_length = pairs
        .get("min_word_length")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);

    let selected_word = pairs.get("word").map(|s| s.to_lowercase());

    let member_id = match pairs.get("member") {
        Some(match_member) => match match_member.parse::<u64>() {
            Ok(member_id) => Some(member_id),
            Err(_) => None,
        },
        None => None,
    };

    let prefix_list: Vec<&str> = vec![
        "$", "&", "!", ".", "m.", ">", "<", "[", "]", "@", "#", "%", "^", "*", ",",
    ];

    let embed = tokio::task::spawn_blocking(move || {
        let conn = Connection::open("messages.db").expect("Unable to open database");

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

        CreateMessage::new().embed(
            CreateEmbed::new()
                .title(format!("Word Leaderboard for {}", guild_id))
                .description(description),
        )
    })
    .await
    .unwrap();

    msg.channel_id.send_message(&ctx.http, embed).await?;
    Ok(())
}
