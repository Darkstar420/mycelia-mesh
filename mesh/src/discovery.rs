use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::sync::Arc;

use futures::StreamExt;
use libp2p::identity;
use libp2p::mdns::{Event as MdnsEvent, tokio::Behaviour as Mdns};
use libp2p::swarm::{Swarm, SwarmEvent};
use libp2p::{Multiaddr, PeerId, multiaddr::Protocol};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::time::Duration;

static SHARDS: Lazy<Arc<RwLock<HashMap<u64, PeerId>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Mesh handles peer discovery via mDNS.
pub struct Mesh {
    peer_id: PeerId,
    peers: Arc<RwLock<HashSet<PeerId>>>,
    addrs: Arc<RwLock<HashMap<PeerId, IpAddr>>>,
    shards: Arc<RwLock<HashMap<u64, PeerId>>>,
    _task: tokio::task::JoinHandle<()>,
}

fn rebalance_inner(
    peers: &Arc<RwLock<HashSet<PeerId>>>,
    shards: &Arc<RwLock<HashMap<u64, PeerId>>>,
    local: PeerId,
) {
    let peers_set = peers.read().clone();
    let mut shard_map = shards.write();
    let mut active: Vec<PeerId> = peers_set.into_iter().collect();
    active.push(local);
    let mut orphaned = Vec::new();
    for (id, owner) in shard_map.iter() {
        if !active.contains(owner) {
            orphaned.push(*id);
        }
    }
    for id in &orphaned {
        shard_map.remove(id);
    }
    if active.is_empty() {
        return;
    }
    for (idx, id) in orphaned.into_iter().enumerate() {
        let peer = active[idx % active.len()];
        shard_map.insert(id, peer);
    }
}

impl Mesh {
    /// Spawn a new mDNS service and return the Mesh instance.
    pub async fn new() -> Self {
        // Generate identity for this node.
        let keypair = identity::Keypair::generate_ed25519();
        let peer_id = keypair.public().to_peer_id();

        // Create transport and mDNS behaviour.
        #[allow(deprecated)]
        let transport = libp2p::tokio_development_transport(keypair).expect("create transport");
        let mdns_cfg = libp2p::mdns::Config {
            ttl: Duration::from_secs(1),
            query_interval: Duration::from_secs(1),
            enable_ipv6: false,
        };
        let behaviour = Mdns::new(mdns_cfg, peer_id).expect("create mdns behaviour");
        let mut swarm = Swarm::new(
            transport,
            behaviour,
            peer_id,
            libp2p::swarm::Config::with_tokio_executor(),
        );

        // Listen on any address. Required for mDNS to broadcast our presence.
        swarm
            .listen_on(
                Multiaddr::empty()
                    .with(Protocol::Ip4([0, 0, 0, 0].into()))
                    .with(Protocol::Tcp(0)),
            )
            .expect("start listener");

        let peers = Arc::new(RwLock::new(HashSet::new()));
        let addrs = Arc::new(RwLock::new(HashMap::new()));
        let peers_task = peers.clone();
        let addrs_task = addrs.clone();
        let shards_task = SHARDS.clone();

        let task = tokio::spawn(async move {
            loop {
                match swarm.next().await {
                    Some(SwarmEvent::Behaviour(MdnsEvent::Discovered(list))) => {
                        let mut set = peers_task.write();
                        let mut map = addrs_task.write();
                        for (peer, addr) in list {
                            set.insert(peer);
                            if let Some(ip) = addr.iter().find_map(|p| match p {
                                Protocol::Ip4(ip) => Some(IpAddr::V4(ip)),
                                Protocol::Ip6(ip) => Some(IpAddr::V6(ip)),
                                _ => None,
                            }) {
                                map.insert(peer, ip);
                            }
                        }
                    }
                    Some(SwarmEvent::Behaviour(MdnsEvent::Expired(list))) => {
                        let mut set = peers_task.write();
                        let mut map = addrs_task.write();
                        for (peer, _addr) in list {
                            set.remove(&peer);
                            map.remove(&peer);
                        }
                        drop(set);
                        drop(map);
                        rebalance_inner(&peers_task, &shards_task, peer_id);
                    }
                    Some(_) => {}
                    None => break,
                }
            }
        });

        Mesh {
            peer_id,
            peers,
            addrs,
            shards: SHARDS.clone(),
            _task: task,
        }
    }

    /// Return the set of discovered peers.
    pub fn peers(&self) -> HashSet<PeerId> {
        self.peers.read().clone()
    }

    /// Return mapping of peer IDs to their IP addresses.
    pub fn addresses(&self) -> HashMap<PeerId, IpAddr> {
        self.addrs.read().clone()
    }

    /// Return this node's peer ID.
    pub fn local_peer_id(&self) -> PeerId {
        self.peer_id
    }

    /// Insert or update a shard assignment.
    pub fn insert_shard(&self, shard_id: u64, peer: PeerId) {
        self.shards.write().insert(shard_id, peer);
    }

    /// Return the current shard assignments.
    pub fn shards(&self) -> HashMap<u64, PeerId> {
        self.shards.read().clone()
    }

    /// Rebalance orphaned shards among active peers.
    pub async fn rebalance(&self) {
        rebalance_inner(&self.peers, &self.shards, self.peer_id);
    }
}
