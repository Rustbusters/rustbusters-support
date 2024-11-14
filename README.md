# RustBusters Support Bot

This is a support bot for the RustBusters drone. It is useful for the groups that "bought" the drone and need help with
it or want
to report a bug.

## How to start the bot

To start the bot, you need to have an .env file with the following content:

```
TELOXIDE_TOKEN=your_token_here (ex. 1234567890:ABCdefGhIjKlMnOpQrStUvWxYz)
SUPPORT_GROUP=your_support_group_id (ex. -100123456789)
```

After that, you can run the bot with the following command:

```
cargo run
```