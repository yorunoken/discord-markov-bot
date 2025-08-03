# Discord Markov Bot

A Discord bot that generates entertaining messages using Markov chain algorithms based on your server's message history.

## Features

-   **Message Generation**: Creates new messages by analyzing patterns in your server's chat history
-   **Message Guessing Game**: Interactive game where users guess who wrote random messages
-   **Word Leaderboards**: Track the most frequently used words by server members
-   **Automatic Message Generation**: Periodically posts generated messages to active channels

## Prerequisites

-   Rust 1.81+
-   SQLite (handled automatically)
-   Discord Bot Token

## Installation

1. Clone the repository:

    ```bash
    git clone https://github.com/yorunoken/discord-markov-bot.git
    cd discord-markov-bot
    ```

2. Set up environment variables:

    ```bash
    cp .env.example .env
    # Edit .env and add your Discord bot token
    ```

3. Build the project:
    ```bash
    cargo build --release
    ```

## Usage

Start the bot:

```bash
cargo run --release
```

The bot will automatically:

-   Connect to Discord using your token
-   Create a SQLite database for message storage
-   Begin collecting messages from your server
-   Generate and post new messages periodically

## Commands

-   `/generate [word]` - Generate a Markov chain message, optionally starting with a specific word
-   `/guess` - Start an interactive message guessing game
-   `/leaderboard [options]` - View word usage statistics for your server
-   `/ping` - Check bot responsiveness

## How It Works

1. **Message Collection**: The bot monitors server channels and stores messages in a local SQLite database
2. **Markov Chain Training**: Messages are processed to build probability chains of word sequences
3. **Content Generation**: New messages are created by following the learned probability patterns
4. **Interactive Games**: Users can participate in guessing games using the collected message history

## Database Schema

The bot automatically creates and manages:

-   `messages` table for storing server messages
-   `user_ratings` table for tracking game performance

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## License

Licensed under the Apache License 2.0. See [LICENSE](LICENSE) for details.

## Support

For questions, issues, or feature requests, please open an issue on GitHub.
