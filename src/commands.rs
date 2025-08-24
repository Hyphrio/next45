use crate::prelude::*;
use argh::FromArgs;

mod forty_five;

pub struct Context<T> {
    pub env: Env,
    pub payload: T,
    pub config: CommandConfig,
}

pub trait CallableV2<Payload>: argh::FromArgs {
    async fn call(self, context: Context<Payload>) -> BotResult<Option<String>>;
}

#[derive(argh::FromArgs, Debug)]
pub struct Root {
    #[argh(subcommand)]
    pub sub: Subcommands,
}

#[derive(argh::FromArgs, Debug)]
#[argh(subcommand)]
pub enum Subcommands {
    FortyFive(forty_five::FortyFiveBot),
}

pub async fn parse(env: Env, payload: ChannelChatMessageV1Payload) {
    console_log!("{:?}", &payload.message.fragments);

    let args = payload
        .message
        .text
        .split_whitespace()
        // 7tv sends an invalid unicode on chromium so drop the
        // &str that has the specific character
        .filter(|x| *x != "\u{e0000}")
        .collect::<Vec<_>>();

    if !args[0].starts_with("!") {
        return;
    }

    if payload.chatter_user_id == UserId::from_static(BOT_USER_ID) {
        return;
    }

    if let Some(shared_chat_channel_id) = &payload.source_broadcaster_user_id
        && shared_chat_channel_id != &payload.broadcaster_user_id
    {
        return;
    }

    let run = Root::from_args(&[], &args);

    match run {
        Ok(root) => {
            let config_binding = env
                .kv(CONFIG_KV)
                .expect("Failed to open configuration values");

            let config = config_binding
                .get(payload.broadcaster_user_id.as_str())
                .json()
                .await
                .ok()
                .flatten()
                .unwrap_or_default();

            let context = Context {
                env: env.clone(),
                payload: payload.clone(),
                config,
            };

            match root.sub {
                Subcommands::FortyFive(forty_five_bot) => {
                    let resp = match forty_five_bot.sub.unwrap_or_default() {
                        forty_five::Subcommands::Gen(generate) => generate.call(context).await,
                        forty_five::Subcommands::Best(best) => best.call(context).await,
                        forty_five::Subcommands::Worst(worst) => worst.call(context).await,
                        forty_five::Subcommands::Pb(pb) => pb.call(context).await,
                        forty_five::Subcommands::Pw(pw) => pw.call(context).await,
                        forty_five::Subcommands::Hof(hof) => hof.call(context).await,
                        forty_five::Subcommands::Timeout(timeout) => timeout.call(context).await,
                        forty_five::Subcommands::Untimeout(untimeout) => {
                            untimeout.call(context).await
                        }
                    };

                    let scoped: Result<(), BotError> = async move {
                        let Some(resp) = resp? else {
                            // No message, just ignore.
                            return Ok(());
                        };

                        // Twitch-related things
                        let token = get_credentials(&env).await?;
                        let http_client = HelixClient::with_client(FetchClient::default());

                        http_client
                            .send_chat_message(
                                &payload.broadcaster_user_id,
                                UserId::from_static(BOT_USER_ID),
                                &*resp,
                                &token,
                            )
                            .await?;

                        Ok(())
                    }
                    .await;

                    if let Err(e) = scoped {
                        console_error!("Error processing command: {:?}", e);
                    }
                }
            }
        }
        Err(e) => {
            console_debug!("{:?}", e);
        }
    }
}
