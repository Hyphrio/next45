use serde::{Deserialize, Serialize};

/// User ID of the bot for it to be able to ignore it's own messages.
pub const BOT_USER_ID: &str = "1179987305";

/// A KV for storing the bot's own credentials.
pub const CREDENTIALS_KV: &str = "Credentials";

/// The access token name from CREDENTIALS_KV.
pub const CREDENTIALS_ACCESS_TOKEN: &str = "tw_access_token";

/// A KV for storing timeout-related data.
pub const TIMEOUTS_KV: &str = "Timeouts";

/// A KV for storing broadcaster-specific configuration.
pub const CONFIG_KV: &str = "BroadcasterConfiguration";

/// A D1 database for storing !45 data.
pub const FORTYFIVE_DATA: &str = "DB";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommandConfig {
    pub forty_five: FortyFiveConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct FortyFiveConfig {
    /// The message that's emitted when a chatter gets 45.000. Such messages would be like this:
    /// `{{ chatter_user_name }} has achieved perfect 45!`, `{{ chatter_user_name }}` would be
    /// dynamically replaced by the actual chatter emitted by Twitch when sending the event.
    pub perfect_45_message: Option<String>,
}
