use anyhow::{Ok, Result};
use serde_json::json;
use tokio::io::{self, AsyncBufReadExt};

async fn request(message: &str, avatar_url: &str, username: &str) -> Result<()> {

    let embed = json!([
        {
            "title": "a",
            "type": "rich",
            "description": "aa"
        }
    ]);
    
    let payload = json!({
        "content": message,
        "avatar_url": avatar_url,
        "username": username,
        "embeds": embed
    });

    let client = reqwest::Client::new();
    let res = client.post("https://discord.com/api/webhooks/1350248908445712435/MPGunb1_eGlZQk7G1stwl0aG6bfU6zV3TpOSiLzAJT1thhNvxepNnqMO1ckyC_0OfFq6")
            .json(&payload)            
            .send()
            .await?;
    println!("{}", res.text().await?);
    return Ok(());
}

async fn user_input() {
    let mut avatar_url = String::new();
    let mut username = String::new();

    let mut customize = String::new();
    let mut cin = io::BufReader::new(io::stdin());
    println!("Customize Webhook? y/n");
    let _ = cin.read_line(&mut customize).await;

    match customize.trim() {
        "y" | "Y" => {
            println!("avatar url for webhook");
            let mut stdin = io::BufReader::new(io::stdin());
            let _ = stdin.read_line(&mut avatar_url).await;

            println!("username for webhook");
            let mut uin = io::BufReader::new(io::stdin());
            let _ = uin.read_line(&mut username).await;

            let _ = request("test", &avatar_url.as_str(), &username).await;
        }
        "n" | "N" => {
            let _ = request("test", "", "").await;
        }
        _ => {
            println!("Invalid input. Please enter 'y' or 'n'.");
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    user_input().await;
    Ok(())
}
