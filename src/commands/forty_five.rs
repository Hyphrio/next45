use rand::Rng;

use crate::{
    commands::{
        CallableV2,
        forty_five::timeouts::{TimeoutData, timeout_key},
    },
    prelude::*,
};

mod best_worst;
mod hof;
mod timeouts;

#[derive(argh::FromArgs, Debug)]
#[argh(subcommand, name = "!45")]
pub struct FortyFiveBot {
    #[argh(subcommand)]
    pub sub: Option<Subcommands>,
}

#[derive(argh::FromArgs, Debug)]
#[argh(subcommand)]
pub enum Subcommands {
    // Commands usable by anyone.
    Gen(Generate),
    Best(best_worst::Best),
    Worst(best_worst::Worst),
    Pb(best_worst::PersonalBest),
    Pw(best_worst::PersonalWorst),
    Hof(hof::HallOfFame),
    // Moderator-only command
    Timeout(timeouts::Timeout),
    Untimeout(timeouts::Untimeout),
}

impl Default for Subcommands {
    fn default() -> Self {
        Self::Gen(Generate {})
    }
}

#[derive(argh::FromArgs, Debug)]
#[argh(subcommand, name = "gen")]
pub struct Generate {}

impl CallableV2<ChannelChatMessageV1Payload> for Generate {
    async fn call(
        self,
        context: super::Context<ChannelChatMessageV1Payload>,
    ) -> BotResult<Option<String>> {
        use rust_decimal::prelude::*;

        let timeouts = context.env.kv(TIMEOUTS_KV)?;

        let db = context.env.d1(FORTYFIVE_DATA)?;
        let db_conn = sqlx_d1::D1Connection::new(db);

        let chatter_timeout: Option<TimeoutData> = timeouts
            .get(&timeout_key(
                &context.payload.broadcaster_user_id,
                Some(&context.payload.chatter_user_id),
            ))
            .json()
            .await?;

        if let Some(_data) = chatter_timeout {
            // The user is timed out and such dont generate !45s.
            return Ok(None);
        }

        let mut rng = rand::rng();

        let time: i64 = web_time::SystemTime::now()
            .duration_since(web_time::SystemTime::UNIX_EPOCH)?
            .as_millis()
            .try_into()?;

        let raw_45 = rng.random_range(dec!(0)..=dec!(18000));
        let refined_45 = raw_45 * dec!(0.005);
        let difference = if refined_45 >= dec!(45.005) {
            refined_45 - dec!(45)
        } else {
            dec!(45) - refined_45
        };

        let message = if raw_45 == dec!(9000) {
            let message = context
                .config
                .forty_five
                .perfect_45_message
                .unwrap_or_else(|| "{{ chatter_user_name }} has achieved perfect 45!".to_owned());

            message.replace(
                "{{ chatter_user_name }}",
                context.payload.chatter_user_name.as_str(),
            )
        } else {
            format!("{}, {}", context.payload.chatter_user_name, refined_45)
        };

        sqlx_d1::query(
            "
            WITH EpochCTE AS (
                SELECT COUNT(*) AS epoch
                FROM Attempts
                WHERE forty_five_difference = 0 AND broadcaster_user_id  = ?1
            )
            INSERT INTO Attempts (epoch, broadcaster_user_id, chatter_user_id, forty_five_value, forty_five_difference, forty_five_timestamp)
            SELECT epoch, ?1, ?2, ?3, ?4, ?5 FROM EpochCTE;
            "
        )
        .bind(context.payload.broadcaster_user_id.as_str())
        .bind(context.payload.chatter_user_id.as_str())
        .bind(refined_45.to_f64().expect("Failed to convert decimal to f64"))
        .bind(difference.to_f64().expect("Failed to convert decimal to f64"))
        .bind(time)
        .execute(&db_conn)
        .await?;

        Ok(Some(message))
    }
}
