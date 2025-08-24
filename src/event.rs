use twitch_api::eventsub::{Event, Message, Payload};
use worker::*;

pub fn verify_signature(
    key: impl AsRef<[u8]>,
    input: impl AsRef<[u8]>,
    expected: &[u8; 32],
) -> bool {
    let signature = hmac_sha256::HMAC::mac(input, key);

    constant_time_eq::constant_time_eq_32(&signature, expected)
}

pub async fn event(env: Env, ctx: Context, event: Event) -> Result<Response> {
    match event {
        // channel.chat.message: Notification
        Event::ChannelChatMessageV1(Payload {
            message: Message::Notification(msg),
            ..
        }) => {
            ctx.wait_until(crate::commands::parse(env, msg));
        }
        // channel.chat.message: Payload verification
        Event::ChannelChatMessageV1(Payload {
            message: Message::VerificationRequest(ver),
            ..
        }) => {
            return Ok(Response::builder()
                .with_header("Content-Type", "text/plain")?
                .with_status(200)
                .body(ResponseBody::Body(ver.challenge.into_bytes())));
        }
        _ => (),
    }

    Response::empty()
}
