use anyhow::Result;
use libp2p::{identity, Swarm};
use libp2p_gossipsub::{Behaviour, IdentTopic, MessageAuthenticity};
use quick_error::ResultExt;

use crate::NodeConfig;

pub async fn new_swarm(config: &NodeConfig, topic: &IdentTopic) -> Result<Swarm<Behaviour>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = local_key.public().to_peer_id();
    println!("My local id is {local_peer_id:?}");

    let message_authenticity = MessageAuthenticity::Signed(local_key.clone());
    let transport = libp2p::tokio_development_transport(local_key)?;
    let gossipsub_config = libp2p_gossipsub::Config::default();
    let mut behavior: libp2p_gossipsub::Behaviour =
        libp2p_gossipsub::Behaviour::new(message_authenticity, gossipsub_config)
            .context("Unable to create gossipsub behavior")
            .unwrap();
    let _subscription = behavior.subscribe(&topic).unwrap();

    // Create a Swarm to manage peers and events
    let mut swarm =
        libp2p::swarm::SwarmBuilder::with_tokio_executor(transport, behavior, local_peer_id)
            .build();

    let port = &config.port.clone().unwrap_or("0".into());

    // Tell the swarm to listen on all interfaces and a random, OS-assigned port.
    let _addr = swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", port).parse()?)?;

    Ok(swarm)
}
