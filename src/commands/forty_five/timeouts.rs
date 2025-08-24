use serde::{Deserialize, Serialize};

use crate::{
    commands::{CallableV2, Context},
    prelude::*,
};

#[derive(argh::FromArgs, Debug)]
#[argh(subcommand, name = "timeout")]
pub struct Timeout {
    #[argh(positional)]
    pub chatter_user_login: String,
    #[argh(positional, default = "300")]
    pub secs: u64,
}

#[derive(argh::FromArgs, Debug)]
#[argh(subcommand, name = "untimeout")]
pub struct Untimeout {
    #[argh(positional)]
    pub chatter_user_login: String,
}

impl CallableV2<ChannelChatMessageV1Payload> for Timeout {
    async fn call(
        self,
        context: crate::commands::Context<ChannelChatMessageV1Payload>,
    ) -> BotResult<Option<String>> {
        timeout_impl(
            context,
            TimeoutAction::Timeout { secs: self.secs },
            self.chatter_user_login,
        )
        .await
    }
}

impl CallableV2<ChannelChatMessageV1Payload> for Untimeout {
    async fn call(
        self,
        context: crate::commands::Context<ChannelChatMessageV1Payload>,
    ) -> BotResult<Option<String>> {
        timeout_impl(context, TimeoutAction::Untimeout, self.chatter_user_login).await
    }
}

enum TimeoutAction {
    Timeout { secs: u64 },
    Untimeout,
}

async fn timeout_impl(
    context: Context<ChannelChatMessageV1Payload>,
    action: TimeoutAction,
    target: String,
) -> BotResult<Option<String>> {
    let http_client = HelixClient::with_client(FetchClient::default());
    let credentials = get_credentials(&context.env).await?;
    let timeouts = context.env.kv(TIMEOUTS_KV)?;

    let is_mod = context
        .payload
        .badges
        .iter()
        .any(|x| x.set_id.as_str() == "moderator" || x.set_id.as_str() == "broadcaster");

    if !is_mod {
        // The user is not a moderator and such the messages should be ignored.
        return Ok(None);
    }

    let chatter = target.replace("@", "");

    let user = http_client
        .get_user_from_login(&chatter.to_lowercase(), &credentials)
        .await?;

    let Some(user) = user else {
        return Ok(Some(format!("User {} not found.", &chatter)));
    };

    match action {
        TimeoutAction::Timeout { secs } => {
            let secs = if secs < 60 { 60 } else { secs };

            let timestamp: i64 = web_time::SystemTime::now()
                .duration_since(web_time::SystemTime::UNIX_EPOCH)?
                .as_millis()
                .try_into()?;

            timeouts
                .put(
                    &timeout_key(&context.payload.broadcaster_user_id, Some(&user.id)),
                    TimeoutData { timestamp, secs },
                )?
                .expiration_ttl(secs)
                .execute()
                .await?;

            Ok(Some(format!(
                "Timed out {} from !45's for {} seconds.",
                chatter, secs
            )))
        }
        TimeoutAction::Untimeout => {
            let timeout_key = timeout_key(&context.payload.broadcaster_user_id, Some(&user.id));

            let timed_out_user: Option<TimeoutData> = timeouts.get(&timeout_key).json().await?;

            let msg = if timed_out_user.is_some() {
                timeouts.delete(&timeout_key).await?;
                format!("Removed !45 timeout for {}.", chatter)
            } else {
                format!("{} is not currently timed out.", chatter)
            };

            Ok(Some(msg))
        }
    }
}

pub fn timeout_key(broadcaster_user_id: &UserId, chatter_user_id: Option<&UserId>) -> String {
    format!(
        "broadcaster={};chatter={}",
        broadcaster_user_id.as_str(),
        match chatter_user_id {
            Some(id) => id.as_str(),
            None => "*",
        },
    )
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeoutData {
    pub timestamp: i64,
    pub secs: u64,
}
