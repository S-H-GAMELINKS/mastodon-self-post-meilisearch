use megalodon::{generator, streaming::Message, entities};
use std::env;
use tokio;
use dotenv::dotenv;
use env_logger;
use meilisearch_sdk::client::*;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use futures::executor::block_on;

#[derive(Serialize, Deserialize, Debug)]
struct IndexStatus {
    id: String,
    content: String,
    created_at: DateTime<Utc>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    env_logger::init();

    let Ok(url) = env::var("MASTODON_STREAMING_URL") else {
        println!("Specify MASTODON_STREAMING_URL!!");
        return
    };
    let Ok(token) = env::var("MASTODON_ACCESS_TOKEN") else {
        println!("Specify MASTODON_ACCESS_TOKEN!!");
        return
    };

    streaming(url.as_str(), token).await;
}

async fn add_index(status: entities::Status) {
    let Ok(url) = env::var("MEILISEARCH_URL") else {
        println!("Specify MEILISEARCH_URL!!");
        return
    };
    let Ok(key) = env::var("MEILISEARCH_API_KEY") else {
        println!("Specify MEILISEARCH_API_KEY!!");
        return
    };

    let client = Client::new(url, Some(key));

    let index = client.index("statuses");

    index.add_documents(&[
        IndexStatus {
            id: status.id,
            content: status.content,
            created_at: status.created_at
        },
    ], Some("id")).await.unwrap();
}

async fn streaming<'cn>(url: &str, access_token: String) {
    let client = generator(
        megalodon::SNS::Mastodon,
        url.to_string(),
        Some(access_token),
        None,
    );
    let streaming = client.user_streaming(url.to_string());

    streaming
        .listen(Box::new(|message| match message {
            Message::Update(mes) => block_on(async move {
                if mes.account.acct == "S_H_" {
                    add_index(mes).await;
                }
            }),
            Message::Notification(mes) => {
                println!("{:#?}", mes);
            }
            Message::Conversation(mes) => {
                println!("{:#?}", mes);
            }
            Message::Delete(mes) => {
                println!("message is deleted: {}", mes);
            }
            Message::StatusUpdate(mes) => {
                println!("updated: {:#?}", mes)
            }
            Message::Heartbeat() => {
                println!("heartbeat");
            }
        }))
        .await;
}