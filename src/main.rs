pub mod status_strem_status_provider;
use reqwest::Client;
#[tokio::main]
async fn main() {
   println!("Hello, World!");
}
async fn https(){
    let client:Client = reqwest::ClientBuilder::new().build().unwrap();
    let res = client
        .get("https://example.com")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
}