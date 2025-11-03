use crate::commands::{CallableV2, Context};
use crate::prelude::*;

#[derive(argh::FromArgs, Debug)]
#[argh(subcommand, name = "best")]
pub struct Best {}

#[derive(argh::FromArgs, Debug)]
#[argh(subcommand, name = "pb")]
pub struct PersonalBest {
    #[argh(positional)]
    pub chatter_user_name: Option<String>,
}

#[derive(argh::FromArgs, Debug)]
#[argh(subcommand, name = "worst")]
pub struct Worst {}

#[derive(argh::FromArgs, Debug)]
#[argh(subcommand, name = "pw")]
pub struct PersonalWorst {
    #[argh(positional)]
    pub chatter_user_name: Option<String>,
}

impl CallableV2<ChannelChatMessageV1Payload> for Best {
    async fn call(
        self,
        context: crate::commands::Context<ChannelChatMessageV1Payload>,
    ) -> BotResult<Option<String>> {
        best_worst_impl(false, false, None, context).await
    }
}

impl CallableV2<ChannelChatMessageV1Payload> for Worst {
    async fn call(
        self,
        context: crate::commands::Context<ChannelChatMessageV1Payload>,
    ) -> BotResult<Option<String>> {
        best_worst_impl(false, true, None, context).await
    }
}

impl CallableV2<ChannelChatMessageV1Payload> for PersonalBest {
    async fn call(
        self,
        context: crate::commands::Context<ChannelChatMessageV1Payload>,
    ) -> BotResult<Option<String>> {
        best_worst_impl(true, false, self.chatter_user_name, context).await
    }
}

impl CallableV2<ChannelChatMessageV1Payload> for PersonalWorst {
    async fn call(
        self,
        context: crate::commands::Context<ChannelChatMessageV1Payload>,
    ) -> BotResult<Option<String>> {
        best_worst_impl(true, true, self.chatter_user_name, context).await
    }
}

async fn best_worst_impl(
    is_personal: bool,
    is_worst: bool,
    chatter_user_name: Option<String>,
    context: Context<ChannelChatMessageV1Payload>,
) -> BotResult<Option<String>> {
    // Database related init
    let database = context.env.d1(FORTYFIVE_DATA)?;
    let db_conn = sqlx_d1::D1Connection::new(database);

    // Twitch-related things
    let token = get_credentials(&context.env).await?;
    let http_client = HelixClient::with_client(FetchClient::default());

    // Personal Bests and Worsts
    let user_id = if let Some(login) = &chatter_user_name {
        let user = http_client
            .get_user_from_login(&login.replace("@", "").to_lowercase(), &token)
            .await?;

        match user {
            Some(user) => {
                let id = user.id.as_str().to_owned();

                (Some(id), user.display_name.as_str().to_owned())
            }
            None => return Ok(Some(format!("User {login} not found."))),
        }
    } else {
        (
            Some(context.payload.chatter_user_id.as_str().to_owned()),
            context.payload.chatter_user_name.as_str().to_owned(),
        )
    };

    let result = sqlx_d1::query!(
        "
        SELECT chatter_user_id, forty_five_value
        FROM Attempts
        WHERE
            Attempts.epoch = (SELECT COUNT(*) FROM Attempts WHERE forty_five_difference = 0 AND broadcaster_user_id = ?3)
            AND Attempts.broadcaster_user_id = ?3
            AND (?1 = 0 OR Attempts.chatter_user_id = ?4)
        ORDER BY
            (CASE
                WHEN ?2 = 0 THEN
                    +Attempts.forty_five_difference
                ELSE
                    -Attempts.forty_five_difference
            END),
            Attempts.forty_five_timestamp DESC
        LIMIT 1;
        ",
        is_personal,
        is_worst,
        context.payload.broadcaster_user_id.as_str(),
        user_id.0
    ).fetch_one(&db_conn).await;

    let query = match result {
        Ok(query) => query,
        Err(error) => match error {
            sqlx_d1::Error::RowNotFound => {
                let query = sqlx_d1::query!(
                    "
                    SELECT
                      chatter_user_id
                    FROM
                      Attempts
                    WHERE
                      broadcaster_user_id = ?1
                      AND chatter_user_id = ?2
                    LIMIT 1;
                    ",
                    context.payload.broadcaster_user_id.as_str(),
                    user_id.0
                )
                .fetch_one(&db_conn)
                .await;

                if query.is_ok() {
                    return Ok(Some(format!(
                        "User {} has done a !45 in this channel, but a perfect 45 has been achieved and such the values has been wiped.",
                        user_id.1
                    )));
                }

                if let Some(login) = &chatter_user_name {
                    return Ok(Some(format!(
                        "User {} hasn't done a !45 in this channel.",
                        login
                    )));
                }

                return Ok(None);
            }
            others => return Err(others.into()),
        },
    };

    if let Some(user) = http_client
        .get_user_from_id(&query.chatter_user_id, &token)
        .await?
    {
        let resp = format!(
            "{} 45 by {}: {:.3}",
            if !is_personal {
                if !is_worst {
                    "Current best"
                } else {
                    "Current worst"
                }
            } else if !is_worst {
                "Personal best"
            } else {
                "Personal worst"
            },
            user.display_name.as_str(),
            query.forty_five_value
        );

        return Ok(Some(resp));
    }

    Ok(None)
}
