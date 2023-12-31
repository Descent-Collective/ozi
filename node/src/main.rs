mod config;

use std::thread;
use std::time::Duration;

use anyhow::Result;
use colored::Colorize;
use config_file::FromConfigFile;
use ozi::{CryptoCompare, Exchange, PriceServiceClient};
use structopt::StructOpt;
use tarpc::{client, context, tokio_serde::formats::Json};
use tracing::Instrument;

use config::NodeConfig;

#[derive(Debug, StructOpt)]
#[structopt(name = "ozi-client", about = "ozi client usage.")]
pub struct Opt {
    #[structopt(short = "c", long = "config", default_value = "node_config.toml")]
    config_path: String,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    match client_impl(opt).await {
        Ok(_) => {}
        Err(err) => {
            for cause in err.chain() {
                println!("{}", cause.to_string().red());
            }
        }
    }
}

async fn client_impl(opt: Opt) -> Result<()> {
    let config = NodeConfig::from_config_file(opt.config_path.clone())?;

    // libp2p stuff - we may not need anymore
    //
    // let id_keys = identity::Keypair::generate_ed25519();
    // let topic = libp2p_gossipsub::IdentTopic::new(config.topic.clone());
    // let mut swarm =
    // ozi::new_swarm(id_keys.clone(), config.port, config.bootstrap_nodes, &topic).await?;

    let addr = tokio::net::lookup_host(config.relayer_address)
        .await?
        .collect::<Vec<_>>();

    let addr = addr.get(0).unwrap();

    let mut transport = tarpc::serde_transport::tcp::connect(addr, Json::default);
    transport.config_mut().max_frame_length(usize::MAX);

    let client = PriceServiceClient::new(client::Config::default(), transport.await?).spawn();

    // todo: use config.exchange_origin to select exchange
    let cc_client = CryptoCompare::new();

    loop {
        // libp2p stuff - we may not need anymore
        //
        // match swarm.select_next_some().await {
        //     SwarmEvent::NewListenAddr { address, .. } => {
        //         println!("Listening on {address:?}");
        //         swarm
        //             .behaviour_mut()
        //             .kademlia
        //             .add_address(&id_keys.public().to_peer_id(), address);
        //     }
        //     SwarmEvent::Behaviour(event) => match event {
        //         ozi::Event::Kademlia(event) => match event {
        //             libp2p::kad::Event::RoutingUpdated {
        //                 peer,
        //                 is_new_peer,
        //                 addresses,
        //                 ..
        //             } => {
        //                 println!("Routing updated {peer} {is_new_peer} {addresses:?}");
        //                 for address in addresses.iter() {
        //                     swarm.dial(address.clone())?;
        //                 }
        //             }
        //             _ => {}
        //         },
        //         ozi::Event::GossipSub(event) => match event {
        //             libp2p_gossipsub::Event::Message {
        //                 propagation_source,
        //                 message,
        //                 ..
        //             } => {
        //                 let data = String::from_utf8_lossy(message.data.as_slice());
        //                 let peer = propagation_source;
        //                 println!("{peer:?} sent: {data:?}");
        //             }
        //             libp2p_gossipsub::Event::Subscribed { peer_id, topic } => {
        //                 let peer = peer_id;
        //                 println!("{peer:?} subscribed to: {topic:?}");
        //             }
        //             _ => {}
        //         },
        //         ozi::Event::Identify(event) => match event {
        //             libp2p::identify::Event::Received { peer_id, info } => {
        //                 if info
        //                     .protocols
        //                     .iter()
        //                     .any(|p| p.clone() == libp2p::kad::PROTOCOL_NAME)
        //                 {
        //                     for addr in info.listen_addrs {
        //                         swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
        //                     }
        //                 }
        //             }
        //             _ => {}
        //         },
        //         _ => {}
        //     },
        //     _ => {}
        // }

        let (encoded_prices, signature) = cc_client
            .run(config.collateral_pairs.clone(), config.private_key.clone())
            .await;

        let rpc_responses = async {
            tokio::select! {
                add_prices_response = client.add_prices(context::current(), encoded_prices.clone(), signature.clone()) => { add_prices_response }
            }
        }
        .instrument(tracing::info_span!("Two Hellos"))
        .await;

        match rpc_responses {
            Ok(response) => {
                println!("{}", response);
                thread::sleep(Duration::from_millis(config.interval));
            }
            Err(e) => tracing::warn!("{:?}", anyhow::Error::from(e)),
        }

        // libp2p stuff - we may not need anymore
        //
        // let serialized = serde_json::to_string(&(encoded_prices, signature))
        //     .context("Unable to serialize result")?;
        // let publish = swarm
        //     .behaviour_mut()
        //     .gossipsub
        //     .publish(topic.clone(), serialized.as_bytes());
        // if publish.is_ok() {
        //     thread::sleep(Duration::from_millis(config.interval));
        // }
    }
}
