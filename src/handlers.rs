// handlers.rs
use crate::commands::Command;
use crate::state::{support_group_id, StateContainer};
use crate::util::{get_random_topic_color, get_user_name};
use std::sync::Arc;
use teloxide::sugar::request::RequestReplyExt;
use teloxide::{
    prelude::*,
    types::{ChatKind, MessageKind},
};

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

            let user_id = msg.chat.id;
            let topic_name = format!("Support-{}", get_user_name(&msg.from.clone().unwrap()));

            // Save the user who requested the topic
            let mut pending_chat = state.pending_chat.lock().await;
            *pending_chat = Some(user_id);

            // Create new forum topic
            bot.create_forum_topic(
                support_group_id(),
                &topic_name,
                get_random_topic_color(),
                "New support ticket",
            )
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
        if let Some(from) = &msg.from {
            if from.is_bot {
                let mut pending_chat = state.pending_chat.lock().await;
                if let Some(chat_id) = *pending_chat {
                    let mut bindings = state.bindings.lock().await;
                    bindings.insert(chat_id, msg.id);
                    *pending_chat = None;

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
