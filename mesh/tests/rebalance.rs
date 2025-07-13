use std::time::Duration;

use mesh::discovery::Mesh;
use tokio::time::Instant;

#[tokio::test(flavor = "multi_thread")]
async fn shard_rebalances_on_exit() {
    let mesh_a = Mesh::new().await;
    let mesh_b = Mesh::new().await;
    let mesh_c = Mesh::new().await;

    let id_a = mesh_a.local_peer_id();
    let id_b = mesh_b.local_peer_id();
    let id_c = mesh_c.local_peer_id();

    // wait for discovery
    let start = Instant::now();
    loop {
        let peers_b = mesh_b.peers();
        let peers_c = mesh_c.peers();
        if peers_b.contains(&id_a)
            && peers_c.contains(&id_a)
            && peers_b.contains(&id_c)
            && peers_c.contains(&id_b)
        {
            break;
        }
        if start.elapsed() > Duration::from_secs(5) {
            panic!("peers not discovered in time");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    mesh_a.insert_shard(1, id_a);

    drop(mesh_a);

    let start = Instant::now();
    loop {
        mesh_b.rebalance().await;
        mesh_c.rebalance().await;
        let shards_b = mesh_b.shards();
        let shards_c = mesh_c.shards();
        if let Some(p) = shards_b.get(&1) {
            if *p != id_a {
                assert!(*p == id_b || *p == id_c);
                break;
            }
        }
        if let Some(p) = shards_c.get(&1) {
            if *p != id_a {
                assert!(*p == id_b || *p == id_c);
                break;
            }
        }
        if start.elapsed() > Duration::from_secs(5) {
            panic!("shard not rebalanced in time");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
