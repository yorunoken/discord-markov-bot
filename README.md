# discord-markov-bot

`discord-markov-bot` is a Discord bot designed to give you and your friends a laugh by combining messages from different times!

It uses the Markov chain to generate new messages.

## Getting Started

### Installation

1. **Clone the repository:**

    ```sh
    git clone https://github.com/yorunoken/discord-markov-bot.git
    cd discord-markov-bot
    ```

2. **Build:**

    ```sh
    cargo build
    ```


### Environment variables

1. **Update env file:**

    Edit the `.env.example` and delete the `.example` off of it.

    ```
    DISCORD_TOKEN=
    ```

    The names are self-explanatory.

### Usage

To start the bot, run:

```sh
cargo run
```

### Compiling for aarch64

1. **Download and install cross**
    navigate to [cross's github repository](https://github.com/cross-rs/cross) and follow instructions

2. **Use the aarch64-unknown-linux-gnu target**
    ```sh
    cross build -r --target aarch64-unknown-linux-gnu
    ```

### About

**The bot will:**

1. Create a database for message storage
3. Read messages from channels.
4. Store the messages in the database.
5. Periodically generate new messages using the Markov chain model.
6. Post the generated messages the chat.

**Commands:**

There are no commands yet, but I'm working on them!

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

For any questions or support, please open an issue on this repository.
