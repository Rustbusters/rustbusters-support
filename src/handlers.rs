// handlers.rs
use crate::commands::Command;
use crate::state::{support_group_id, Language, StateContainer};
use crate::util::{get_random_topic_color, get_user_name};
use std::sync::Arc;
use teloxide::sugar::request::RequestReplyExt;
use teloxide::{
    prelude::*,
    types::{ChatKind, InlineKeyboardButton, InlineKeyboardMarkup, MessageKind},
};

const CALLBACK_ITALIAN: &str = "lang_it";
const CALLBACK_ENGLISH: &str = "lang_en";

/// Creates an inline keyboard for language selection
fn create_language_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("🇮🇹 Italiano", CALLBACK_ITALIAN),
        InlineKeyboardButton::callback("🇬🇧 English", CALLBACK_ENGLISH),
    ]])
}

/// Handles bot commands
pub async fn handle_commands(
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
            // Verify command was sent in private chat
            if !matches!(msg.chat.kind, ChatKind::Private(_)) {
                bot.send_message(msg.chat.id, "This command can only be used in private chat")
                    .await?;
                return Ok(());
            }

            // Check if user already has an open ticket
            let bindings = state.bindings.lock().await;
            if bindings.contains_key(&msg.chat.id) {
                bot.send_message(msg.chat.id, "You already have an open support ticket. Close it with /close or write a new message.")
                  .await?;
                return Ok(());
            }
            drop(bindings);

            // Check if there's already a pending request
            let pending_chat = state.pending_chat.lock().await;
            if pending_chat.is_some() {
                bot.send_message(msg.chat.id, "Another support request is being processed. Please wait a moment and try again.")
                  .await?;
                return Ok(());
            }
            drop(pending_chat);

            // Send language selection message
            let keyboard = create_language_keyboard();
            bot.send_message(
                msg.chat.id,
                "Please select your preferred language for support:",
            )
            .reply_markup(keyboard)
            .await?;
        }
        Command::Close => {
            let mut bindings = state.bindings.lock().await;
            if let Some(&topic_msg_id) = bindings.get(&msg.chat.id) {
                // Close topic by replying to the creating message
                bot.send_message(
                    support_group_id(),
                    format!(
                        "Chat ended by the user {}",
                        get_user_name(&msg.from.clone().unwrap())
                    ),
                )
                .reply_to(topic_msg_id)
                .await?;

                bot.send_message(msg.chat.id, "The support topic has been closed.")
                    .await?;

                bindings.remove(&msg.chat.id);
            } else if msg.chat.id == support_group_id() {
                if let Some(reply_to) = msg.reply_to_message() {
                    if let Some((&private_chat_id, _)) =
                        bindings.iter().find(|(_, &msg_id)| msg_id == reply_to.id)
                    {
                        bot.send_message(support_group_id(), "Chat ended")
                            .reply_to(reply_to.id)
                            .await?;
                        bot.send_message(private_chat_id, "RustBusters closed the support chat. Write /support to open a new one.")
                          .await?;
                    }
                }

                bindings.retain(|_, &mut msg_id| msg_id != msg.id);
            }
        }
    }
    Ok(())
}

/// Handles regular messages
pub async fn handle_messages(
    bot: Bot,
    msg: Message,
    state: Arc<StateContainer>,
) -> Result<(), teloxide::RequestError> {
    // Handle topic creation
    if let MessageKind::ForumTopicCreated(_) = &msg.kind {
        // Verify if the topic was created by the bot
        if let Some(from) = &msg.from {
            if from.is_bot {
                let mut pending_chat = state.pending_chat.lock().await;
                if let Some((chat_id, language)) = *pending_chat {
                    // Save the binding
                    let mut bindings = state.bindings.lock().await;
                    bindings.insert(chat_id, msg.id);

                    // Clear pending_chat
                    *pending_chat = None;

                    // Send confirmation message
                    let confirmation = match language {
                        Language::Italian => "Ticket di supporto creato! Puoi ora chattare con RustBusters attraverso questo bot. Per chiudere la chat, usa /close.",
                        Language::English => "Support ticket created! You can now chat with RustBusters through this bot. To close the chat, use /close.",
                    };
                    bot.send_message(chat_id, confirmation).await?;
                }
            }
        }
        return Ok(());
    }

    let bindings = state.bindings.lock().await;

    match msg.chat.kind {
        // Handle private chat messages
        ChatKind::Private(_) => {
            if let Some(&topic_msg_id) = bindings.get(&msg.chat.id) {
                if let Some(text) = msg.text() {
                    bot.send_message(support_group_id(), text)
                        .reply_to(topic_msg_id)
                        .await?;
                }
            }
        }
        // Handle forum messages
        ChatKind::Public(_) => {
            if msg.chat.id == support_group_id() {
                if let Some(reply_to) = msg.reply_to_message() {
                    if let Some((&private_chat_id, _)) =
                        bindings.iter().find(|(_, &msg_id)| msg_id == reply_to.id)
                    {
                        if let Some(text) = msg.text() {
                            bot.send_message(private_chat_id, text).await?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Handles callback queries (inline keyboard buttons)
pub async fn handle_callback_query(
    bot: Bot,
    query: CallbackQuery,
    state: Arc<StateContainer>,
) -> Result<(), teloxide::RequestError> {
    println!("ciao");
    if let (Some(data), Some(message)) = (&query.data, &query.message) {
        let language = match data.as_str() {
            CALLBACK_ITALIAN => Some(Language::Italian),
            CALLBACK_ENGLISH => Some(Language::English),
            _ => None,
        };

        if let (Some(language), from) = (language, &query.from) {
            // Store the user and selected language
            let mut pending_chat = state.pending_chat.lock().await;
            *pending_chat = Some((message.chat().id, language));

            // Delete the language selection message
            bot.delete_message(message.chat().id, message.id()).await?;

            // Create the topic with the appropriate language suffix
            let topic_name = format!("{} • {}", language.to_flag(), get_user_name(from));
            bot.create_forum_topic(
                support_group_id(),
                &topic_name,
                get_random_topic_color(),
                "New support ticket",
            )
            .await?;
        }
    }

    // Answer the callback query to remove the loading state
    bot.answer_callback_query(&query.id).await?;

    Ok(())
}
