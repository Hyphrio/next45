use crate::commands::CallableV2;
use crate::prelude::*;

#[derive(argh::FromArgs, Debug)]
#[argh(subcommand, name = "hof")]
pub struct HallOfFame {
    #[argh(positional)]
    pub epoch: Option<usize>,
}

impl CallableV2<ChannelChatMessageV1Payload> for HallOfFame {
    async fn call(
        self,
        context: crate::commands::Context<ChannelChatMessageV1Payload>,
    ) -> BotResult<Option<String>> {
        let http_client = HelixClient::with_client(FetchClient::default());
        let credentials = get_credentials(&context.env).await?;

        let database = context.env.d1(FORTYFIVE_DATA)?;
        let db_conn = sqlx_d1::D1Connection::new(database);

        let query = sqlx_d1::query!(
            "
            SELECT
              chatter_user_id
            FROM Attempts  
            WHERE 
              forty_five_value = 45.000
              AND broadcaster_user_id = ?1
              AND (?2 = 0 OR epoch = ?3)
            ORDER BY
              epoch DESC
            LIMIT 1
            ",
            &context.payload.broadcaster_user_id.as_str(),
            self.epoch.is_some(),
            self.epoch
        )
        .fetch_one(&db_conn)
        .await;

        match query {
            Ok(query) => {
                if let Some(user) = http_client
                    .get_user_from_id(&query.chatter_user_id, &credentials)
                    .await?
                {
                    let msg = match self.epoch {
                        Some(epoch) => {
                            format!("Perfect 45.000 #{} by: {}", epoch, user.display_name)
                        }
                        None => format!("Latest perfect 45.000 by: {}", user.display_name),
                    };

                    return Ok(Some(msg));
                }

                Ok(None)
            }
            Err(err) => {
                return match err {
                    sqlx_d1::Error::RowNotFound => {
                        let msg = match self.epoch {
                            Some(value) => {
                                format!("No perfect 45.000's found with epoch of {}", value)
                            }
                            None => "No perfect 45.000's in this channel.".to_owned(),
                        };
                        Ok(Some(msg))
                    }
                    _ => Ok(None),
                };
            }
        }
    }
}
