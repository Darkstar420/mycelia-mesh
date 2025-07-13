use std::collections::HashSet;
use std::sync::Arc;

use futures::StreamExt;
use libp2p::identity;
use libp2p::mdns::{Event as MdnsEvent, tokio::Behaviour as Mdns};
use libp2p::swarm::{Swarm, SwarmEvent};
use libp2p::{Multiaddr, PeerId, multiaddr::Protocol};
use parking_lot::RwLock;

/// Mesh handles peer discovery via mDNS.
pub struct Mesh {
    peer_id: PeerId,
    peers: Arc<RwLock<HashSet<PeerId>>>,
    _task: tokio::task::JoinHandle<()>,
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
        let behaviour = Mdns::new(Default::default(), peer_id).expect("create mdns behaviour");
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
        let peers_task = peers.clone();

        let task = tokio::spawn(async move {
            loop {
                match swarm.next().await {
                    Some(SwarmEvent::Behaviour(MdnsEvent::Discovered(list))) => {
                        let mut set = peers_task.write();
                        for (peer, _addr) in list {
                            set.insert(peer);
                        }
                    }
                    Some(SwarmEvent::Behaviour(MdnsEvent::Expired(list))) => {
                        let mut set = peers_task.write();
                        for (peer, _addr) in list {
                            set.remove(&peer);
                        }
                    }
                    Some(_) => {}
                    None => break,
                }
            }
        });

        Mesh {
            peer_id,
            peers,
            _task: task,
        }
    }

    /// Return the set of discovered peers.
    pub fn peers(&self) -> HashSet<PeerId> {
        self.peers.read().clone()
    }

    /// Return this node's peer ID.
    pub fn local_peer_id(&self) -> PeerId {
        self.peer_id
    }
}
