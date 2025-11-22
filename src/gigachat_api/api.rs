use std::fs;
use std::time::{SystemTime, Duration};
use async_trait::async_trait;
use log::debug;
use reqwest::{Certificate, Client, Url};
use tokio::sync::Mutex;
use uuid::Uuid;
use tracing::{error, info, warn, instrument};
use crate::gigachat_api::dto::{GigaChatAuthRequest, GigaChatAuthResponse, GigaChatGenerateTextRequest, GigaChatGenerateTextResponse, GigaChatMessage, GigaChatRole};
use crate::errors::ApiError;
use crate::errors::ApiError::{ApiClientBuildError, ApiStatusError, CertParseError, DecodeResponseError, NoContent, RequestError};
use crate::generation_controller::ContentRephraser;
#[derive(Debug)]
pub struct GigaChatApi {
    pub server: Client,
    client_id: String,
    client_secret: String,
    access_token: Mutex<String>,
    access_token_expire_at: Mutex<SystemTime>
}

impl GigaChatApi {
    pub fn new(client_id: String, client_secret: String) -> Result<Self, ApiError> {
        let cert_pem = fs::read("cert.crt")?;

        let cert = Certificate::from_pem(&cert_pem)
            .map_err(CertParseError)?;

        let custom_client = Client::builder()
            .add_root_certificate(cert)
            .build()
            .map_err(ApiClientBuildError)?;

        Ok(GigaChatApi{
            server: custom_client,
            client_id,
            client_secret,
            access_token: Mutex::new(String::new()),
            access_token_expire_at: Mutex::new(SystemTime::UNIX_EPOCH)
        })
    }

    #[instrument(skip(self), err)]
    async fn refresh_auth_token(&self) -> Result<(), ApiError> {
        info!("Starting to refresh token");

        let auth_refresh_url =
            Url::parse("https://ngw.devices.sberbank.ru:9443/api/v2/oauth")?;

        let raw_req = GigaChatAuthRequest {
            scope: "GIGACHAT_API_PERS".to_string(),
        };

        let response = self.server
            .post(auth_refresh_url)
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .header("RqUID", Uuid::new_v4().to_string())
            .form(&raw_req)
            .send()
            .await
            .map_err(RequestError)?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!(%status, %body, "Failed to refresh token");
            return Err(ApiStatusError { model: "gigachat".to_string(), status, body });
        }

        let resp_text = response.text()
            .await
            .map_err(DecodeResponseError)?;

        let auth_response: GigaChatAuthResponse = serde_json::from_str(&resp_text)?;

        let mut access_token_guard = self.access_token.lock().await;
        let mut access_token_expire_at_guard = self.access_token_expire_at.lock().await;

        *access_token_guard = auth_response.access_token;
        *access_token_expire_at_guard = auth_response.expires_at;

        info!("Successfully refreshed token");
        Ok(())
    }
}

#[async_trait]
impl ContentRephraser for GigaChatApi {
    #[instrument(skip(self, current_text), err)]
    async fn rephrase_text(&self, current_text: &str) -> Result<String, ApiError> {
        info!("Starting to rephrase text");

        let is_expired = {
            let expire_at = self.access_token_expire_at.lock().await;
            *expire_at <= (SystemTime::now() + Duration::from_secs(3))
        };

        if is_expired {
            self.refresh_auth_token().await?
        }

        let system_message = GigaChatMessage::new_system_message();
        let message_to_rephrase = GigaChatMessage::new(GigaChatRole::User,
                                                       current_text.to_string());
        let request = GigaChatGenerateTextRequest {
            model: "GigaChat-2".to_string(),
            messages: vec![system_message, message_to_rephrase],
        };

        for attempt in 1..= 2 {
            debug!("{} {}", attempt, "Sending generation request...");

            let auth_header = {
                let current_access_token = self.access_token.lock().await;
                format!("Bearer {}", current_access_token)
            };

            let generate_content_url =
                Url::parse("https://gigachat.devices.sberbank.ru/api/v1/chat/completions")?;

            debug!("Sending generation request to GigaChat...");
            let response = self.server
                .post(generate_content_url)
                .header(reqwest::header::AUTHORIZATION, auth_header)
                .json(&request)
                .send()
                .await
                .map_err(RequestError)?;

            if response.status() == reqwest::StatusCode::UNAUTHORIZED {
                if attempt == 1 {
                    info!("Refreshing token and retrying...");
                    self.refresh_auth_token().await?;
                    continue;
                } else {
                    let body = response.text().await.unwrap_or_default();
                    error!(%body, "Failed to authenticate even after refresh");
                    return Err(
                        ApiStatusError{
                            model: "gigachat".to_string(),
                            status: reqwest::StatusCode::UNAUTHORIZED,
                            body
                        }
                    );
                }
            }


            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                error!(%status, %body, "GigaChat generation failed");
                return Err(ApiStatusError { model: "gigachat".to_string(), status, body });
            }

            let resp_text = response.text().await.map_err(DecodeResponseError)?;
            let response: GigaChatGenerateTextResponse = serde_json::from_str(&resp_text)?;

            if let Some(new_text) = response.choices.into_iter().next() {
                info!("Text rephrased successfully");
                return Ok(new_text.message.content)
            }
        }

        warn!("GigaChat returned 200 OK but empty choices");
        Err(NoContent)
    }
}