use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::sync::OnceLock;
use teloxide::types::{ChatId, MessageId};
use tokio::sync::Mutex;

#[derive(Clone, Copy, PartialEq)]
pub enum Language {
    Italian,
    English,
}

impl Language {
    pub fn to_flag(&self) -> &'static str {
        match self {
            Language::Italian => "🇮🇹",
            Language::English => "🇬🇧",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum TicketType {
    Bug,
    HowTo,
    Other,
}

impl TicketType {
    pub fn to_string(&self) -> String {
        match self {
            TicketType::Bug => "Bug",
            TicketType::HowTo => "How to ...",
            TicketType::Other => "Other",
        }
        .to_string()
    }
}

/// Global support group chat ID
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

/// Container for the application state
#[derive(Clone)]
pub struct StateContainer {
    /// Maps private ChatId to topic MessageId
    pub bindings: Arc<Mutex<HashMap<ChatId, MessageId>>>,
    /// Stores the ChatId, selected language and ticket type of the user who requested the last topic
    pub pending_chat: Arc<Mutex<Option<(ChatId, Language, Option<TicketType>)>>>,
}

impl StateContainer {
    pub fn new() -> Self {
        Self {
            bindings: Arc::new(Mutex::new(HashMap::new())),
            pending_chat: Arc::new(Mutex::new(None)),
        }
    }
}
