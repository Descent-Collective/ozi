use anyhow::Result;

use libp2p::{identity, Swarm};
use libp2p_gossipsub::{IdentTopic, MessageAuthenticity};

use crate::behavior::Behaviour;
use crate::NodeConfig;

pub async fn new_swarm(
    id_keys: identity::Keypair,
    config: &NodeConfig,
    topic: &IdentTopic,
) -> Result<Swarm<Behaviour>> {
    let local_peer_id = id_keys.public().to_peer_id();

    println!("My peer id is {local_peer_id:?}");

    let mut behavior = Behaviour::new(local_peer_id, id_keys.clone(), config)
        .expect("should be able to create behavior");

    let _subscription = behavior.gossipsub.subscribe(&topic).unwrap();

    // Create a Swarm to manage peers and events
    let transport = libp2p::tokio_development_transport(id_keys)?;
    let mut swarm =
        libp2p::swarm::SwarmBuilder::with_tokio_executor(transport, behavior, local_peer_id)
            .build();

    let port = &config.port.clone().unwrap_or("0".into());

    // Tell the swarm to listen on all interfaces and a random, OS-assigned port.
    let _addr = swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", port).parse()?)?;

    Ok(swarm)
}
