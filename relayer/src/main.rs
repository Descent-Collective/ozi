use relayer::RelayerClient;

use colored::Colorize;
use structopt::StructOpt;

use futures::{future, prelude::*};
use ozi::PriceService;
use std::net::{IpAddr, Ipv6Addr};
use tarpc::{
    server::{self, incoming::Incoming, Channel},
    tokio_serde::formats::Json,
};

#[derive(Debug, StructOpt)]
#[structopt(name = "relayer-client", about = "relayer client usage.")]
struct Opt {
    #[structopt(long)]
    port: u16,
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

async fn main_impl(opt: Opt) -> anyhow::Result<()> {
    let server_addr = (IpAddr::V6(Ipv6Addr::LOCALHOST), opt.port);
    let mut listener = tarpc::serde_transport::tcp::listen(&server_addr, Json::default).await?;
    listener.config_mut().max_frame_length(usize::MAX);

    println!("Listening on port {}", listener.local_addr().port());

    listener
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        .max_channels_per_key(1, |t| t.transport().peer_addr().unwrap().ip())
        .map(|channel| {
            let server =
                RelayerClient::new_with_socket_addr(channel.transport().peer_addr().unwrap());
            channel.execute(server.serve())
        })
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;

    Ok(())
}
