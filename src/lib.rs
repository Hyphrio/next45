//! A Twitch bot that generates a number between 0 and 90, with 0.005 increments.
//! Simulates a 45 strafe in Minecraft.
//!
//! If you want to use the bot for your own purposes, change the variables defined in consts.rs,
//! adopting it to your account. Change the KV and D1 values as well as these define which KV store
//! and databases the bot uses. You must also define 3 secrets which are TW_CLIENT_ID
//! (the Twitch API Client ID), TW_CLIENT_SECRET (the client secret to use) and HMAC_SECRET
//! (the value that the bot checks for when receiving events from Twitch). See the Twitch developer
//! website for more information.
//!
//! You must manually get the bot token for the first time and set it in the credentials KV. The key
//! for setting the access token is defined by CREDENTIALS_ACCESS_TOKEN in consts.rs.

use twitch_api::eventsub::Event;
use worker::*;

mod commands;
mod config;
mod error;
mod event;
mod prelude;
mod twitch;

#[event(fetch)]
async fn fetch(req: Request, env: Env, ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    Router::with_data(ctx)
        .post_async("/eventsub", eventsub)
        .run(req, env)
        .await
}

async fn eventsub(
    mut req: Request,
    RouteContext { data, env, .. }: RouteContext<Context>,
) -> Result<Response> {
    let body = req.text().await;

    let Ok(body) = body else {
        return Ok(Response::builder()
            .with_status(400)
            .body(ResponseBody::Empty));
    };

    let Some(message_id) = req.headers().get("twitch-eventsub-message-id")? else {
        return Ok(Response::builder()
            .with_status(400)
            .body(ResponseBody::Empty));
    };

    let Some(timestamp) = req.headers().get("twitch-eventsub-message-timestamp")? else {
        return Ok(Response::builder()
            .with_status(400)
            .body(ResponseBody::Empty));
    };

    let Some(signature) = req.headers().get("twitch-eventsub-message-signature")? else {
        return Ok(Response::builder()
            .with_status(400)
            .body(ResponseBody::Empty));
    };

    let mut buf = [0u8; 32];
    let Ok(_) = hex::decode_to_slice(signature.replace("sha256=", ""), &mut buf) else {
        return Ok(Response::builder()
            .with_status(400)
            .body(ResponseBody::Empty));
    };

    let key = env.secret("HMAC_SECRET")?;
    let mut input = String::with_capacity(message_id.len() + timestamp.len() + body.len());
    input.push_str(&message_id);
    input.push_str(&timestamp);
    input.push_str(&body);

    if event::verify_signature(key.to_string(), input, &buf) {
        let parse_event = Event::parse(&body);

        if let Ok(event) = parse_event {
            return event::event(env, data, event).await;
        }
    }

    Ok(Response::builder()
        .with_status(400)
        .body(ResponseBody::Empty))
}
