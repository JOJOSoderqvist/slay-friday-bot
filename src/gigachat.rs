use std::fs;
use std::time::{SystemTime, Duration};
use reqwest::{Certificate, Client, Url};
use tokio::sync::Mutex;
use uuid::Uuid;
use crate::dto::{GigaChatAuthRequest, GigaChatAuthResponse, GigaChatGenerateTextRequest, GigaChatGenerateTextResponse, GigaChatMessage};
use crate::errors::ApiError;
use crate::errors::ApiError::NoContent;

pub struct GigaChatApi {
    pub server: Client,
    client_id: String,
    client_secret: String,
    access_token: Mutex<String>,
    access_token_expire_at: Mutex<SystemTime>
}

impl GigaChatApi {
    pub fn new(client_id: String, client_secret: String) -> Self {
        let cert_pem = fs::read("cert.crt")
            .expect("Failed to read the certificate file. Make sure 'russian_trusted_root_ca.pem' is in the project root.");

        let cert = Certificate::from_pem(&cert_pem)
            .expect("Failed to create a certificate from the PEM file.");

        let custom_client = Client::builder()
            .add_root_certificate(cert)
            .build()
            .expect("Failed to build the custom reqwest client.");


        GigaChatApi{
            server: custom_client,
            client_id,
            client_secret,
            access_token: Mutex::new(String::new()),
            access_token_expire_at: Mutex::new(SystemTime::UNIX_EPOCH)
        }
    }

    async fn refresh_auth_token(&self) -> Result<(), ApiError> {
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
            .await?;


        // TODO: Add unsuccessfull resp status checks
        // if !response.status().is_success() {
        //     todo!()
        // }

        let resp_text = response.text().await?;

        let auth_response: GigaChatAuthResponse = serde_json::from_str(&resp_text)?;

        let mut access_token_guard = self.access_token.lock().await;
        let mut access_token_expire_at_guard = self.access_token_expire_at.lock().await;

        *access_token_guard = auth_response.access_token;
        *access_token_expire_at_guard = auth_response.expires_at;

        Ok(())
    }

    pub async fn rephrase_text(&self, current_text: &str) -> Result<String, ApiError> {
        let mut is_expired = false;

        {
            let expire_at = self.access_token_expire_at.lock().await;
            if *expire_at <= (SystemTime::now() + Duration::from_secs(3)) {
                is_expired = true;
            }
        }

        if is_expired {
            self.refresh_auth_token().await?
        }

        let current_access_token = self.access_token.lock().await.clone();

        let generate_content_url =
            Url::parse("https://gigachat.devices.sberbank.ru/api/v1/chat/completions")?;

        let system_message = GigaChatMessage::new_system_message();
        let message_to_rephrase = GigaChatMessage::new("user".to_string(),
                                                       current_text.to_string());
        let request = GigaChatGenerateTextRequest {
            model: "GigaChat-2".to_string(),
            messages: vec![system_message, message_to_rephrase],
        };

        let response = self.server
            .post(generate_content_url)
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", current_access_token))
            .json(&request)
            .send()
            .await?;

        let resp_text = response.text().await?;
        let response: GigaChatGenerateTextResponse = serde_json::from_str(&resp_text)?;

        if let Some(new_text) = response.choices.get(0) {
            return Ok(new_text.message.content.to_string())
        }

        Err(NoContent)
    }
}