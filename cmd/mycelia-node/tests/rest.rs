use std::time::Duration;

use mycelia_node::{GenerateRequest, Role, run};
use reqwest::Client;

#[tokio::test(flavor = "multi_thread")]
async fn generate_round_robin() {
    let worker1 = tokio::spawn(run(Role::Worker, 11501));
    let worker2 = tokio::spawn(run(Role::Worker, 11502));

    // wait for servers to start and discover each other
    tokio::time::sleep(Duration::from_secs(1)).await;

    let shim = tokio::spawn(run(Role::Shim, 11500));

    tokio::time::sleep(Duration::from_secs(3)).await;

    let client = Client::new();
    let mut resp = client
        .post("http://127.0.0.1:11500/api/generate")
        .json(&GenerateRequest {
            prompt: "2+2=?".into(),
        })
        .send()
        .await
        .unwrap();

    let mut body = String::new();
    while let Some(chunk) = resp.chunk().await.unwrap() {
        body.push_str(&String::from_utf8_lossy(&chunk));
    }

    assert!(body.contains("4"));

    shim.abort();
    worker1.abort();
    worker2.abort();
}
