mod config;

use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use colored::Colorize;
use config::NodeConfig;
use config_file::FromConfigFile;
use ozi::{CryptoCompare, Exchange, PriceServiceClient};
use structopt::StructOpt;
use tarpc::{client, context, tokio_serde::formats::Json};
use tracing::Instrument;

#[derive(Debug, StructOpt)]
#[structopt(name = "ozi-node", about = "ozi node usage.")]
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
                eprintln!("{}", cause.to_string().red());
            }
        }
    }
}

async fn client_impl(opt: Opt) -> Result<()> {
    let config = NodeConfig::from_config_file(opt.config_path.clone())?;

    let addr = tokio::net::lookup_host(config.relayer_address)
        .await?
        .collect::<Vec<_>>();

    let addr = addr.get(0).unwrap();

    let mut transport = tarpc::serde_transport::tcp::connect(addr, Json::default);
    transport.config_mut().max_frame_length(usize::MAX);

    let client = PriceServiceClient::new(client::Config::default(), transport.await?).spawn();

    // TODO: use config.exchange_origin to select exchange
    let cryptocompare = CryptoCompare::new();

    loop {
        let (data, signature) = cryptocompare
            .run(config.collateral_pairs.clone(), config.private_key.clone())
            .await
            .context("unable to ")?;

        let rpc_responses = async {
            tokio::select! {
                add_prices_response = client.add_prices(context::current(), data.clone(), signature.clone()) => { add_prices_response }
            }
        }
        .instrument(tracing::info_span!("Two Hellos"))
        .await;

        match rpc_responses {
            Ok(response) => {
                eprintln!("{}", response);
                thread::sleep(Duration::from_millis(config.interval));
            }
            Err(e) => tracing::warn!("{:?}", anyhow::Error::from(e)),
        }
    }
}
