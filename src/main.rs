mod behavior;
mod swarm;

use std::thread;
use std::time::Duration;

use anyhow::Result;
use colored::Colorize;
use config_file::FromConfigFile;
use futures::StreamExt;
use libp2p::{identity, swarm::SwarmEvent, Multiaddr};
use serde::Deserialize;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "ozi", about = "ozi usage.")]
pub struct Opt {
    #[structopt(short = "c", long = "config", default_value = "node_config.toml")]
    config_path: String,
    #[structopt(short = "m", long = "message")]
    message: Option<String>,
    #[structopt(short = "p", long = "ping")]
    ping: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct NodeConfig {
    topic: String,
    /// Ethereum key to use for signing price messages.
    ethereum_key: Option<String>,
    /// Fetch asset prices
    exchange_orign: Option<String>,
    exchange_key: String,
    /// Specifies the interval in milliseconds between sending price messages.
    interval: u64,
    /// List of pairs to send price messages for
    /// This is the collateral we want to fetch prices for
    collateral_pair: Vec<String>,
    /// Port to listen on
    port: Option<String>,
    /// Bootstrap nodes for peer discovery with kademlia protocol
    bootstrap_nodes: Vec<String>,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    match main_impl(opt).await {
        Ok(_) => {}
        Err(err) => {
            for cause in err.chain() {
                println!("{}", cause.to_string().red());
            }
        }
    }
}

async fn main_impl(opt: Opt) -> Result<()> {
    let id_keys = identity::Keypair::generate_ed25519();

    let config = NodeConfig::from_config_file(opt.config_path.clone())?;
    let topic = libp2p_gossipsub::IdentTopic::new(config.topic.clone());
    let mut swarm = swarm::new_swarm(id_keys.clone(), &config, &topic).await?;

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {address:?}");
                swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&id_keys.public().to_peer_id(), address);
            }
            SwarmEvent::Behaviour(event) => match event {
                behavior::Event::Ping(_) => todo!(),
                behavior::Event::Kademlia(event) => match event {
                    libp2p::kad::KademliaEvent::InboundRequest { request } => todo!(),
                    libp2p::kad::KademliaEvent::OutboundQueryProgressed {
                        id,
                        result,
                        stats,
                        step,
                    } => todo!(),
                    libp2p::kad::KademliaEvent::RoutingUpdated {
                        peer,
                        is_new_peer,
                        addresses,
                        bucket_range,
                        old_peer,
                    } => {
                        println!("Routing updated {peer} {is_new_peer} {addresses:?}");
                        for address in addresses.iter() {
                            swarm.dial(address.clone())?;
                        }
                    }
                    libp2p::kad::KademliaEvent::UnroutablePeer { peer } => todo!(),
                    libp2p::kad::KademliaEvent::RoutablePeer { peer, address } => todo!(),
                    libp2p::kad::KademliaEvent::PendingRoutablePeer { peer, address } => todo!(),
                },
                behavior::Event::GossipSub(event) => match event {
                    libp2p_gossipsub::Event::Message {
                        propagation_source,
                        message,
                        ..
                    } => {
                        let data = String::from_utf8_lossy(message.data.as_slice());
                        let peer = propagation_source;
                        println!("{peer:?} sent: {data:?}");
                    }
                    libp2p_gossipsub::Event::Subscribed { peer_id, topic } => {
                        let peer = peer_id;
                        println!("{peer:?} subscribed to: {topic:?}");
                    }
                    libp2p_gossipsub::Event::Unsubscribed { peer_id, topic } => todo!(),
                    libp2p_gossipsub::Event::GossipsubNotSupported { peer_id } => todo!(),
                },
                behavior::Event::Identify(event) => match event {
                    libp2p::identify::Event::Received { peer_id, info } => {
                        if info
                            .protocols
                            .iter()
                            .any(|p| p.clone() == libp2p::kad::PROTOCOL_NAME)
                        {
                            for addr in info.listen_addrs {
                                swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                            }
                        }
                    }
                    libp2p::identify::Event::Sent { peer_id } => todo!(),
                    libp2p::identify::Event::Pushed { peer_id } => todo!(),
                    libp2p::identify::Event::Error { peer_id, error } => todo!(),
                },
            },
            _ => {}
        }
        if let Some(addr) = opt.ping.clone() {
            let remote: Multiaddr = addr.parse()?;
            swarm.dial(remote)?;
        }
        if let Some(message) = opt.message.clone() {
            let behavior = swarm.behaviour_mut();
            let publish = behavior
                .gossipsub
                .publish(topic.clone(), message.as_bytes());
            if publish.is_ok() {
                thread::sleep(Duration::from_millis(config.interval));
            }
        }
    }
    // Ok(())
}
