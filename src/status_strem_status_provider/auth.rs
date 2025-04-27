use reqwest::Client;
use serde::Deserialize;

use super::communications::CommunicationsConfig;
#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}
pub async fn get_access_token(config:CommunicationsConfig) -> Result<(String,i64), Box<dyn std::error::Error>> {
    let token_url = format!("{}/realms/{}/protocol/openid-connect/token", config.kc_url, config.kc_realm);
    let client = Client::new();
    let params = [
        ("grant_type", "client_credentials"),
        ("client_id", &config.kc_client_id),
        ("client_secret", &config.kc_client_secret),
    ];
    let res:TokenResponse = serde_json::from_str(client
        .post(token_url)
        .form(&params)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?.as_str()).unwrap();
    Ok((res.access_token,res.expires_in as i64))
}