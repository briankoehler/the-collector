# The Collector
A software system for 'collecting' data from the Riot Games API and notifying
Discord guilds when summoners they follow perfor underwhelming in a game.

This is a complete re-write of the [original int-bot](https://github.com/briankoehler/int-bot)
I wrote a few years. The original implementation, written in Python, gathered and evaluated
1000+ games of data amongst my friends and I. Unfortunately, due to poor design choices, it was
prone to issues and began to break with changes to Riot's API.

On top of the performance improvements that should come with this re-implementation in Rust (though
even that should be negligible given the network I/O), the type system of Rust provides much more safety as well.

## Architecture
A design tenent when constructing this system was modularity; being able to pull pieces apart and
replace them with newer implementations down the line should be possible. Unlike the original
implementation, which used a long script for the core functionality, things are very much split
apart here.

There are two binaries compiled by this project:
1. **The Collector** — Backend service that handles the data collection from the Riot Games API
and inserting into the database. Sends a message to the Int Bot via an NNG IPC socket when data
of a summoner's match is inserted int the database.
2. **Int Bot** - Discord bot that users interface with via slash commands. Commands include
(un)following summoners, retrieving a leaderboard of the top "ints", and statistics about a
followed summoner. Most importantly, though, it listens for messages from the Collector and
will send a message to the relevant Discord guilds if a followed summoner "ints".

## Code Structure
This project consists of a Cargo workspace that defines two binaries (discussed above), and two
libraries that contain shared logic between them (IPC, database queries).

There is a separate Git repository that contains the database schema used for creating the SQLite
database (subject to being merged into here).

## Setup
1. Prior to building the project, some configuration must be done. Configuration is currently
done via environment variables that may be set in a .env file in the project root directory.
```bash
DATABASE_URL="sqlite://path/to/your.db"
DISCORD_TOKEN="YOUR_DISCORD_TOKEN"
RGAPI_KEY="YOUR_RIOT_API_KEY"
RUST_LOG="int_bot=info,the_collector=info"
SQLX_OFFLINE=true
```
2. Make sure that the sqlx-cli is installed, and then run `cargo sqlx prepare` to generate a `.sqlx`
directory.
3. Finally, run `cargo build`

## Cross-compilation
I've been deploying the system on a Raspberry Pi 3 that runs the vanilla 32-bit OS. Rather than
compiling on the Pi itself (which I did once and waited a *very* long time), I set up
cross-compilation on my MacBook Pro (Apple Silicon) through the
[cross tool](https://github.com/cross-rs/cross) using the following steps:
```bash
# 1. Add the rustup target
rustup target add armv7-unknown-linux-gnueabihf
# 2. Install cross
cargo install cross --git https://github.com/cross-rs/cross
# 3. Install Docker Desktop (via web browser)
# 4. Build
cross build --release --target armv7-unknown-linux-gnueabihf
```
