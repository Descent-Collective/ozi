use anyhow::Result;
use libp2p::{identity, noise, yamux, Swarm};
use libp2p_gossipsub::IdentTopic;

use super::behavior::Behaviour;

pub async fn new_swarm(
    id_keys: identity::Keypair,
    port: Option<String>,
    bootstrap_nodes: Vec<String>,
    _topic: &IdentTopic,
) -> Result<Swarm<Behaviour>> {
    let local_peer_id = id_keys.public().to_peer_id();

    println!("My peer id is {local_peer_id:?}");

    // Create a Swarm to manage peers and events
    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(id_keys)
        .with_tokio()
        .with_tcp(
            Default::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|id_keys| {
            Behaviour::new(local_peer_id, id_keys.clone(), bootstrap_nodes)
                .expect("unable to create behavior")
        })?
        .build();

    let port = port.clone().unwrap_or("0".into());

    // Tell the swarm to listen on all interfaces and a random, OS-assigned port.
    let _addr = swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", port).parse()?)?;

    Ok(swarm)
}
