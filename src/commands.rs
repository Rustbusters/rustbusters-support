// commands.rs
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    /// Start the bot
    Start,
    /// Get the current chat ID
    GetId,
    /// Open a new support ticket
    Support,
    /// Close the current support ticket
    Close,
}
