use teloxide::dispatching::UpdateFilterExt;
use teloxide::prelude::*;
use teloxide::types::{ParseMode, Rgb};
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display this text.")]
    Help,
    Create,
}

async fn handle_commands(
    bot: Bot,
    msg: Message,
    cmd: Command,
) -> Result<(), teloxide::RequestError> {
    match cmd {
        Command::Help => {
            println!("{:#?}", msg);
            bot.send_message(
                msg.chat.id,
                "Questo è un bot di esempio. Usa /help per questa descrizione.",
            )
            .await?;
        }
        Command::Create => {
            bot.create_forum_topic(
                ChatId(-1002337455276),
                "Titolo",
                Rgb::from_u32(251),
                "Testo",
            )
            .await?;
        }
    }
    Ok(())
}

async fn handle_messages(bot: Bot, msg: Message) -> Result<(), teloxide::RequestError> {
    println!("Id\n{:#?}", msg.id);
    println!("From\n{:#?}", msg.from);
    println!("Kind\n{:#?}", msg.kind);
    println!("Chat\n{:#?}", msg.chat);

    // if let ForumTopicCreated(_) = msg.kind {
    //     bot.send_message(ChatId(-1002337455276), "Forum topic created")
    //         .reply_to(MessageId(97))
    //         .await?;
    // }
    if let Some(text) = msg.text() {
        bot.send_message(ChatId(698410803), text)
            .parse_mode(ParseMode::Html)
            .await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let bot = Bot::from_env();

    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(handle_commands),
        )
        .branch(dptree::entry().endpoint(handle_messages));

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
