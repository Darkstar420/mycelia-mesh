use std::time::Duration;

use mesh::discovery::Mesh;
use tokio::time::Instant;

#[tokio::test(flavor = "multi_thread")]
async fn peers_discover_each_other() {
    let mesh1 = Mesh::new().await;
    let mesh2 = Mesh::new().await;

    let id1 = mesh1.local_peer_id();
    let id2 = mesh2.local_peer_id();

    let start = Instant::now();
    loop {
        let peers1 = mesh1.peers();
        let peers2 = mesh2.peers();
        if peers1.contains(&id2) && peers2.contains(&id1) {
            break;
        }
        if Instant::now().duration_since(start) > Duration::from_secs(5) {
            panic!("peers not discovered in time");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
