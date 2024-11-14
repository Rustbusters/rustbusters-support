mod util;

use crate::util::{get_random_topic_color, get_user_name};
use dotenv::dotenv;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::sync::OnceLock;
use teloxide::dispatching::UpdateFilterExt;
use teloxide::prelude::*;
use teloxide::sugar::request::RequestReplyExt;
use teloxide::types::{MessageId, MessageKind};
use teloxide::utils::command::BotCommands;
use tokio::sync::Mutex;

pub fn support_group_id() -> ChatId {
    static SUPPORT_GROUP: OnceLock<ChatId> = OnceLock::new();
    *SUPPORT_GROUP.get_or_init(|| {
        ChatId(
            env::var("SUPPORT_GROUP")
                .expect("SUPPORT_GROUP must be set.")
                .parse()
                .unwrap(),
        )
    })
}

// Struttura per memorizzare i binding tra chat private e message_id del topic
#[derive(Clone)]
struct StateContainer {
    // HashMap che mappa ChatId privato -> MessageId del topic
    bindings: Arc<Mutex<HashMap<ChatId, MessageId>>>,
    // Chat ID dell'utente che ha richiesto l'ultimo topic
    pending_chat: Arc<Mutex<Option<ChatId>>>,
}

impl StateContainer {
    fn new() -> Self {
        Self {
            bindings: Arc::new(Mutex::new(HashMap::new())),
            pending_chat: Arc::new(Mutex::new(None)),
        }
    }
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    GetId,
    Support,
    Close,
}

async fn handle_commands(
    bot: Bot,
    msg: Message,
    cmd: Command,
    state: Arc<StateContainer>,
) -> Result<(), teloxide::RequestError> {
    match cmd {
        Command::GetId => {
            bot.send_message(msg.chat.id, msg.chat.id.0.to_string())
                .await?;
        }
        Command::Support => {
            // Verifica che il comando sia stato inviato in chat privata
            if !matches!(msg.chat.kind, teloxide::types::ChatKind::Private(_)) {
                bot.send_message(msg.chat.id, "This command can only be used in private chat")
                    .await?;
                return Ok(());
            }

            let user_id = msg.chat.id;
            let topic_name = format!("Support-{}", get_user_name(&msg.from.clone().unwrap()));

            // Salva l'utente che ha richiesto il topic
            let mut pending_chat = state.pending_chat.lock().await;
            *pending_chat = Some(user_id);

            // Crea un nuovo topic nel forum
            bot.create_forum_topic(
                support_group_id(),
                &(topic_name),
                get_random_topic_color(),
                "New support ticket",
            )
            .await?;
        }
        Command::Close => {
            let mut bindings = state.bindings.lock().await;
            if let Some(&topic_msg_id) = bindings.get(&msg.chat.id) {
                // Chiudi il topic rispondendo al messaggio che l'ha creato
                bot.send_message(
                    support_group_id(),
                    format!(
                        "Chat ended by the user {}",
                        get_user_name(&msg.from.clone().unwrap())
                    ),
                )
                .reply_to(topic_msg_id)
                .await?;
                // Invia un messaggio di avviso nella chat privata
                bot.send_message(msg.chat.id, "The support topic has been closed.")
                    .await?;

                // Rimuovi il binding
                bindings.remove(&msg.chat.id);
            } else if msg.chat.id == support_group_id() {
                // Cerca il topic a cui il messaggio sta rispondendo
                if let Some(reply_to) = msg.reply_to_message() {
                    if let Some((&private_chat_id, _)) =
                        bindings.iter().find(|(_, &msg_id)| msg_id == reply_to.id)
                    {
                        // Chiudi il topic rispondendo al messaggio che l'ha creato
                        bot.send_message(support_group_id(), "Chat ended")
                            .reply_to(reply_to.id)
                            .await?;
                        // Invia un messaggio di avviso nella chat privata
                        bot.send_message(private_chat_id, "RustBusters closed the support chat. Write /support to open a new one.")
                            .await?;
                    }
                }

                // Rimuovi il binding
                bindings.retain(|_, &mut msg_id| msg_id != msg.id);
            }
        }
    }
    Ok(())
}

async fn handle_messages(
    bot: Bot,
    msg: Message,
    state: Arc<StateContainer>,
) -> Result<(), teloxide::RequestError> {
    // Gestione creazione topic
    if let MessageKind::ForumTopicCreated(_) = &msg.kind {
        // Verifica se il topic è stato creato dal bot stesso
        if let Some(from) = &msg.from {
            if from.is_bot {
                let mut pending_chat = state.pending_chat.lock().await;
                if let Some(chat_id) = *pending_chat {
                    // Salva il binding
                    let mut bindings = state.bindings.lock().await;
                    bindings.insert(chat_id, msg.id);

                    // Pulisci pending_chat
                    *pending_chat = None;

                    // Notifica l'utente
                    bot.send_message(
                        chat_id,
                        "Support ticket created! You can now chat with RustBusters through this bot. To close the chat, use /close.",
                    )
                    .await?;
                }
            }
        }
        return Ok(());
    }

    let bindings = state.bindings.lock().await;

    match msg.chat.kind {
        // Gestione messaggi in chat privata
        teloxide::types::ChatKind::Private(_) => {
            if let Some(&topic_msg_id) = bindings.get(&msg.chat.id) {
                if let Some(text) = msg.text() {
                    // Inoltra il messaggio al topic rispondendo al messaggio che l'ha creato
                    bot.send_message(support_group_id(), text)
                        .reply_to(topic_msg_id)
                        .await?;
                }
            }
        }
        // Gestione messaggi nel forum
        teloxide::types::ChatKind::Public(_) => {
            if msg.chat.id == support_group_id() {
                // Se il messaggio è una risposta ad un topic che abbiamo tracciato
                if let Some(reply_to) = msg.reply_to_message() {
                    if let Some((&private_chat_id, _)) =
                        bindings.iter().find(|(_, &msg_id)| msg_id == reply_to.id)
                    {
                        if let Some(text) = msg.text() {
                            // Inoltra il messaggio alla chat privata
                            bot.send_message(private_chat_id, text).await?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let bot = Bot::from_env();

    // Crea una nuova istanza di StateContainer
    let state = Arc::new(StateContainer::new());

    // Crea la mappa delle dipendenze
    let mut deps = DependencyMap::new();
    deps.insert(state);

    // Crea l'albero delle dipendenze
    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(handle_commands),
        )
        .branch(dptree::entry().endpoint(handle_messages));

    // Costruisci e avvia il dispatcher con le dipendenze
    Dispatcher::builder(bot, handler)
        .dependencies(deps)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
