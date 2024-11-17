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

## How does the bot work?

The bot listens for command `/support` in the private chat with the bot. When the command is received, the bot will
create a new topic in the support group indicated in the `SUPPORT_GROUP` environment variable.

> **Note:**
> After the `/support` command is sent, the user will receive a message for language selection for the support chat.

The bot saves the bindings between the user and the topic in a map. When the user sends a message to the bot, the bot
will forward the message to the topic indicated in the map.

To close the "ticket" the user needs to send the `/close` command to the bot. The bot will remove the binding between
the user and the topic and will send a message to the user indicating that the ticket was closed.
Also the members of the support group can close the ticket by sending the `/close` in the topic chat.

