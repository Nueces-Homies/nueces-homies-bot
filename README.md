# Nueces Homies Bot
Nueces Homies Bot is the new version of the existing Nueces Homies discord bot (i.e. SGCEventsBot). This is a complete rewrite of the bot into Rust from the original Python. The hope is to keep the UX as similiar as possible so that this is an internals only change. 

With the move to Rust and experience writing the Python version, I want to be able to deliver updates and new features without accidentally breaking existing functionality. Also, I want to be able to more quickly find out when external integrations break. This will require developing testable code that adheres to SOLID principles, which wasn't always true in the Python code base. 

### Why Rust?
Mostly because I wanted to learn it. It's also cool that the executables get built down to a single file that's probably much leaner than shipping a Java, dotnet, node, or Python runtime. Also, Rust's type system and annoying compiler are great, which should help prevent bad code commits. 

## Contributing
Rust and Visual Studio Code should be all you need to get started, tooling wise. To fully run the project though, you'll need a .env file. The .env file should include the following:

```properties
GOOGLE_CREDENTIALS=<base64 encoded credentials.json for Google API>
TWITCH_CLIENT_ID=<client ID from Twitch developer portal>
TWITCH_CLIENT_SECRET=<client secret from Twitch developer portal>
TMDB_KEY=<API key from TMDB>
DISCORD_TOKEN=<Discord bot token from developer portal>

DATABASE_PATH=<path to the sqlite database you want to save to>
GUILD_ID=<id of the Discord server bot will live in>
```

### Automatically generating .env file
You can generate the above file by running the following. You will need to install Azure CLI first and join the Azure subscription to get read permissions.

```sh
az login
cargo build
target/debug/get-config <CONFIG_STORE_NAME> >> .env
```


