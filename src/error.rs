#[derive(thiserror::Error, Debug)]
pub enum BotError {
    #[error("Error from workers: {0}")]
    WorkerError(#[from] worker::Error),
    #[error("Failed sending a request to Twitch API")]
    HelixClientError(#[from] twitch_api::helix::ClientRequestError<worker::Error>),
    #[error("Reading/writing to KV failed")]
    KvError(#[from] worker::kv::KvError),
    #[error("Reading/writing to database failed")]
    SqlxError(#[from] sqlx_d1::Error),
    #[error("Worker time error")]
    TimeError(#[from] web_time::SystemTimeError),
    #[error("Number too big")]
    IntError(#[from] std::num::TryFromIntError),
    #[error("Unimplemented.")]
    #[allow(dead_code)]
    Unimplemented,
}

pub type BotResult<T> = core::result::Result<T, BotError>;
