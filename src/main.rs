// main.rs
mod commands;
mod handlers;
mod state;
mod util;

use crate::handlers::{handle_commands, handle_messages};
use crate::state::StateContainer;
use dotenv::dotenv;
use std::sync::Arc;
use teloxide::{dispatching::UpdateFilterExt, prelude::*};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let bot = Bot::from_env();

    // Initialize application state
    let state = Arc::new(StateContainer::new());

    // Setup dependency injection
    let mut deps = DependencyMap::new();
    deps.insert(state);

    // Create the handler tree
    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<commands::Command>()
                .endpoint(handle_commands),
        )
        .branch(dptree::entry().endpoint(handle_messages));

    // Build and launch the dispatcher
    Dispatcher::builder(bot, handler)
        .dependencies(deps)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
