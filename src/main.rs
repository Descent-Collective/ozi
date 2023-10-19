mod gossipsub;

use std::thread;
use std::time::Duration;

use anyhow::Result;
use colored::Colorize;
use config_file::FromConfigFile;
use futures::StreamExt;
use libp2p::{swarm::SwarmEvent, Multiaddr};
use serde::Deserialize;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
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
    let config = NodeConfig::from_config_file(opt.config_path.clone())?;
    let topic = libp2p_gossipsub::IdentTopic::new(config.topic.clone());
    let mut swarm = gossipsub::new_swarm(&config, &topic).await?;

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {address:?}")
            }
            SwarmEvent::Behaviour(event) => match event {
                libp2p_gossipsub::Event::Message {
                    message,
                    propagation_source,
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
                _ => {}
            },
            SwarmEvent::Dialing {
                peer_id,
                connection_id,
            } => println!("Dialing {peer_id:?} from {connection_id:?}"),
            _ => {}
        }
        if let Some(addr) = opt.ping.clone() {
            let remote: Multiaddr = addr.parse()?;
            swarm.dial(remote)?;
        }
        if let Some(message) = opt.message.clone() {
            let behavior = swarm.behaviour_mut();
            let publish = behavior.publish(topic.clone(), message.as_bytes());
            if publish.is_ok() {
                thread::sleep(Duration::from_millis(config.interval));
            }
        }
    }
    // Ok(())
}
