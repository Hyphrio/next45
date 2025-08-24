use std::io::ErrorKind;

use crate::config::{CREDENTIALS_ACCESS_TOKEN, CREDENTIALS_KV};
use http::{Method as HttpMethod, Response};
use twitch_api::{
    HelixClient,
    client::{BoxedFuture, Bytes, Client},
};
use twitch_oauth2::{AccessToken, AppAccessToken, ClientId, ClientSecret};
use worker::{
    CfProperties, Env, Fetch, Headers, Method, Request, RequestInit,
    send::{SendFuture, SendWrapper},
};

pub async fn get_credentials(env: &Env) -> worker::Result<AppAccessToken> {
    let http_client = HelixClient::with_client(FetchClient::default());
    let client_id = env.secret("TW_CLIENT_ID")?.to_string();
    let client_secret = env.secret("TW_CLIENT_SECRET")?.to_string();

    let credentials = env.kv(CREDENTIALS_KV)?;

    let access_token = credentials.get(CREDENTIALS_ACCESS_TOKEN).text().await?;

    let get_token = match access_token {
        Some(access_token) => AppAccessToken::from_existing(
            &http_client,
            AccessToken::new(access_token),
            None,
            ClientSecret::new(client_secret.clone()),
        )
        .await
        .map_err(|_| ()),
        _ => AppAccessToken::get_app_access_token(
            &http_client,
            ClientId::new(client_id.clone()),
            ClientSecret::new(client_secret.clone()),
            vec![],
        )
        .await
        .map_err(|_| ()),
    };

    let token = match get_token {
        Ok(token) => token,
        Err(_) => {
            let new_token = AppAccessToken::get_app_access_token(
                &http_client,
                ClientId::new(client_id.clone()),
                ClientSecret::new(client_secret.clone()),
                vec![],
            )
            .await
            .map_err(|_| worker::Error::Io(ErrorKind::NetworkUnreachable.into()))?;

            credentials
                .put("tw_access_token", new_token.access_token.as_str())?
                .execute()
                .await?;

            new_token
        }
    };

    Ok(token)
}

#[derive(Default, Clone)]
pub struct FetchClient {
    pub ttl: Option<u32>,
}

impl Client for FetchClient {
    type Error = worker::Error;

    fn req(
        &self,
        request: twitch_api::client::Request,
    ) -> BoxedFuture<'_, Result<twitch_api::client::Response, <Self as Client>::Error>> {
        let method = match *request.method() {
            HttpMethod::GET => Method::Get,
            HttpMethod::POST => Method::Post,
            HttpMethod::PATCH => Method::Patch,
            HttpMethod::PUT => Method::Put,
            HttpMethod::DELETE => Method::Delete,
            HttpMethod::CONNECT => Method::Connect,
            HttpMethod::OPTIONS => Method::Options,
            HttpMethod::TRACE => Method::Trace,
            HttpMethod::HEAD => Method::Head,
            _ => unimplemented!(),
        };

        let headers = request
            .headers()
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|v| (k.as_str(), v)))
            .collect::<Headers>();

        let body = match method {
            Method::Get => None,
            _ => Some(crate::wasm_bindgen::JsValue::from(request.body().to_vec())),
        };

        let request_init = RequestInit {
            method,
            body,
            headers,
            cf: CfProperties {
                cache_ttl: self.ttl,
                ..Default::default()
            },
            ..Default::default()
        };

        let request = Request::new_with_init(&request.uri().to_string(), &request_init)
            .expect("URI is valid");

        Box::pin(async move {
            let mut response =
                SendWrapper::new(SendFuture::new(Fetch::Request(request).send()).await?);

            let mut http_response = Response::builder().status(response.status_code());

            for (k, v) in response.headers() {
                http_response = http_response.header(k, v);
            }

            let body = SendFuture::new(response.bytes()).await?;

            let built_response = http_response
                .body(Bytes::from_owner(body))
                .map_err(|_| worker::Error::Infallible)?;

            Ok(built_response)
        })
    }
}
