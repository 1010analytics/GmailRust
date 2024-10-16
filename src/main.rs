use std::path::PathBuf;
use log::{info, error};
use yup_oauth2::{authenticator::Authenticator, InstalledFlowAuthenticator, InstalledFlowReturnMethod};
use hyper::{Client, Request, Body};
use hyper_rustls::HttpsConnectorBuilder;
use warp::Filter;
use std::{env, sync::Arc};
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use std::error::Error;
use chrono::Utc;
use hyper::body::to_bytes;
use serde_json::Value;
use base64::{decode_config, URL_SAFE}; 
use scraper::{Html, Selector}; 

const SCOPES: &[&str] = &["https://www.googleapis.com/auth/gmail.modify"];
const GMAIL_API_BASE_URL: &str = "https://gmail.googleapis.com/gmail/v1/users/me";

#[derive(Debug, Serialize, Deserialize)]
struct Credentials {
    access_token: String,
    refresh_token: Option<String>,
    expires_at: Option<i64>,
}

struct AsyncGmailHandler {
    filter_email: String,
    current_time: chrono::DateTime<chrono::Utc>,
    base_dir: PathBuf,
    auth: Option<Authenticator<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>>,
    client: Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
}

impl AsyncGmailHandler {
    fn new(filter_email: String) -> Self {
        let base_dir = env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();

        let https = HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_only()
            .enable_http1()
            .build();

        let client = Client::builder().build(https);

        Self {
            filter_email,
            current_time: Utc::now(),
            base_dir,
            auth: None,
            client,
        }
    }

    async fn authenticate_gmail(&mut self) -> Result<(), Box<dyn Error>> {
        let credentials_path = PathBuf::from("credentials.json");
        let token_path = self.base_dir.join("token.json");

        info!("Looking for credentials at: {:?}", credentials_path);
        info!("Loading credentials from {:?}", token_path);

        let secret = yup_oauth2::read_application_secret(&credentials_path).await?;

        let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
            .persist_tokens_to_disk(token_path.clone())
            .build()
            .await?;

        self.auth = Some(auth);
        info!("Authentication successful!");

        Ok(())
    }

    async fn refresh_token_if_needed(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(auth) = &self.auth {
            let token = auth.token(SCOPES).await?;
            if token.is_expired() {
                info!("Token expired, refreshing...");
                self.authenticate_gmail().await?;
            }
        } else {
            self.authenticate_gmail().await?;
        }
        Ok(())
    }

    async fn list_messages(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let auth = self.auth.as_ref().ok_or("Not authenticated")?;
        let token = auth.token(SCOPES).await?;

        let request = Request::builder()
            .method("GET")
            .uri(format!("{}/messages", GMAIL_API_BASE_URL))
            .header("Authorization", format!("Bearer {}", token.token().unwrap_or("")))
            .body(Body::empty())?;

        let response = self.client.request(request).await?;

        let body_bytes = to_bytes(response.into_body()).await?;
        let body_text = String::from_utf8(body_bytes.to_vec()).unwrap();

        println!("Response body: {}", body_text);

        let json_response: Value = serde_json::from_str(&body_text)?;
        let mut message_ids = Vec::new();
        if let Some(messages) = json_response.get("messages") {
            for message in messages.as_array().unwrap_or(&Vec::new()) {
                if let Some(id) = message.get("id").and_then(|id| id.as_str()) {
                    message_ids.push(id.to_string());
                }
            }
        }

        Ok(message_ids)
    }

    async fn get_message(&self, message_id: &str) -> Result<(), Box<dyn Error>> {
        let auth = self.auth.as_ref().ok_or("Not authenticated")?;
        let token = auth.token(SCOPES).await?;

        let request = Request::builder()
            .method("GET")
            .uri(format!("{}/messages/{}", GMAIL_API_BASE_URL, message_id))
            .header("Authorization", format!("Bearer {}", token.token().unwrap_or("")))
            .body(Body::empty())?;

        let response = self.client.request(request).await?;

        let body_bytes = to_bytes(response.into_body()).await?;
        let body_text = String::from_utf8(body_bytes.to_vec()).unwrap();

        println!("Full Message Response: {}", body_text);

        let parsed_response: Value = serde_json::from_str(&body_text)?;
        if let Some(payload) = parsed_response.get("payload") {
            if let Some(parts) = payload.get("parts") {
                for part in parts.as_array().unwrap_or(&Vec::new()) {
                    if let Some(body) = part.get("body") {
                        if let Some(data) = body.get("data") {
                        
                            let decoded = decode_config(data.as_str().unwrap_or(""), URL_SAFE)?;
                            let decoded_string = String::from_utf8(decoded)?;
                            println!("Decoded email body: {}", decoded_string);

                            self.parse_html(&decoded_string);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn parse_html(&self, html_body: &str) {
        let document = Html::parse_document(html_body);
        let selector = Selector::parse("p").unwrap(); 

        for element in document.select(&selector) {
            let text = element.text().collect::<Vec<_>>().join(" ");
            println!("Extracted text: {}", text);
        }
    }

    async fn process_all_messages(&self) -> Result<(), Box<dyn Error>> {
        
        let message_ids = self.list_messages().await?;

        
        for message_id in message_ids {
            if let Err(e) = self.get_message(&message_id).await {
                error!("Failed to fetch message ID {}: {}", message_id, e);
            }
        }

        Ok(())
    }
}

async fn run_oauth_server(auth_handler: Arc<Mutex<AsyncGmailHandler>>) {
    let redirect_route = warp::path!("oauth2" / "callback")
        .map(|| {
            
            "OAuth flow completed. You can close this window."
        });

    warp::serve(redirect_route)
        .run(([127, 0, 0, 1], 61074))
        .await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let filter_email = "example@example.com".to_string();
    let handler = Arc::new(Mutex::new(AsyncGmailHandler::new(filter_email)));

    
    let auth_handler = handler.clone();
    tokio::spawn(async move {
        run_oauth_server(auth_handler).await;
    });

    
    let mut handler_guard = handler.lock().await;
    if let Err(e) = handler_guard.refresh_token_if_needed().await {
        error!("An error occurred during authentication: {}", e);
    }

   
    if let Err(e) = handler_guard.process_all_messages().await {
        error!("An error occurred while processing messages: {}", e);
    }

    Ok(())
}
