use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use anyhow::Result;
use libp2p::identify;
use libp2p::identity;
use libp2p::kad;
use libp2p::kad::store::MemoryStore;
use libp2p::ping;
use libp2p::swarm::NetworkBehaviour;
use libp2p::{Multiaddr, PeerId, StreamProtocol};
use libp2p_gossipsub;

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "Event", event_process = true)]
pub struct Behaviour {
    pub gossipsub: libp2p_gossipsub::Behaviour,
    pub kademlia: kad::Behaviour<MemoryStore>,
}

static NONCE: AtomicUsize = AtomicUsize::new(0);

impl Behaviour {
    pub fn new(
        local_peer_id: PeerId,
        id_keys: identity::Keypair,
        bootstrap_nodes: Vec<String>,
    ) -> Result<Self> {
        let store = MemoryStore::new(local_peer_id.clone());
        let mut kademlia_config = kad::Config::default();
        let protocol = StreamProtocol::new("/xxx/0.1.0");
        kademlia_config.set_protocol_names(vec![protocol]);
        kademlia_config.set_record_ttl(Some(Duration::from_secs(0)));
        kademlia_config.set_provider_record_ttl(Some(Duration::from_secs(0)));

        let mut kademlia =
            kad::Behaviour::with_config(local_peer_id.clone(), store, kademlia_config);

        for addr in bootstrap_nodes {
            let multiaddr: Multiaddr = addr.parse()?;
            if let Some(peer_id) = multiaddr.iter().find_map(|x| match x {
                libp2p::multiaddr::Protocol::P2p(peer_id) => Some(peer_id),
                _ => None,
            }) {
                kademlia.add_address(&peer_id, multiaddr.clone());
                println!("👥 Kademlia bootstraped by peer: {peer_id} with address: {multiaddr}");
            }
        }

        let message_id_fn = |message: &libp2p_gossipsub::Message| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            let nonce = NONCE.fetch_add(1, Ordering::SeqCst);
            let combined = format!("{}{}", s.finish(), nonce);
            libp2p_gossipsub::MessageId::from(combined)
        };
        let libp2p_gossipsub_config = libp2p_gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .validation_mode(libp2p_gossipsub::ValidationMode::Strict)
            .message_id_fn(message_id_fn)
            .build()
            .expect("unable to create config");

        let gossipsub = libp2p_gossipsub::Behaviour::new(
            libp2p_gossipsub::MessageAuthenticity::Signed(id_keys.clone()),
            libp2p_gossipsub_config,
        )
        .expect("unable to create behavior");

        Ok(Self {
            gossipsub,
            kademlia,
        })
    }
}

#[derive(Debug)]
pub enum Event {
    Ping(ping::Event),
    Identify(identify::Event),
    Kademlia(kad::Event),
    GossipSub(libp2p_gossipsub::Event),
}

impl From<ping::Event> for Event {
    fn from(event: ping::Event) -> Self {
        Event::Ping(event)
    }
}

impl From<identify::Event> for Event {
    fn from(event: identify::Event) -> Self {
        Event::Identify(event)
    }
}

impl From<libp2p_gossipsub::Event> for Event {
    fn from(event: libp2p_gossipsub::Event) -> Self {
        Event::GossipSub(event)
    }
}

impl From<kad::Event> for Event {
    fn from(event: kad::Event) -> Self {
        Event::Kademlia(event)
    }
}
