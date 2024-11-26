use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::OnceLock;
use std::{env, fs};
use teloxide::types::{ChatId, MessageId};
use tokio::sync::Mutex;

// Aggiungi derive per serializzazione/deserializzazione
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct SavedBinding {
    pub chat_id: i64,
    pub topic_msg_id: i32,
}

impl StateContainer {
    // Nuova funzione per salvare i bindings su file
    pub async fn save_bindings(&self) -> Result<(), std::io::Error> {
        let bindings = self.bindings.lock().await;
        let saved_bindings: Vec<SavedBinding> = bindings
            .iter()
            .map(|(&chat_id, &topic_msg_id)| SavedBinding {
                chat_id: chat_id.0,
                topic_msg_id: topic_msg_id.0,
            })
            .collect();

        let json = serde_json::to_string_pretty(&saved_bindings)?;
        fs::write("/data/bindings.json", json)
    }

    // Nuova funzione per caricare i bindings da file
    pub fn load_bindings() -> HashMap<ChatId, MessageId> {
        let path = Path::new("/data/bindings.json");
        if !path.exists() {
            println!("No bindings file found, starting with an empty state.");
            return HashMap::new();
        }

        let json = fs::read_to_string(path).unwrap_or_default();
        let saved_bindings: Vec<SavedBinding> = serde_json::from_str(&json).unwrap_or_default();

        saved_bindings
            .into_iter()
            .map(|b| (ChatId(b.chat_id), MessageId(b.topic_msg_id)))
            .collect()
    }

    pub fn new() -> Self {
        Self {
            bindings: Arc::new(Mutex::new(Self::load_bindings())),
            pending_chat: Arc::new(Mutex::new(None)),
        }
    }
}
