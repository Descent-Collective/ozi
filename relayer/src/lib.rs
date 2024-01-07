use std::{net::SocketAddr, time::Duration};

use ethers::{abi::ParamType, types::Bytes};
use ozi::{Price, PriceService};
use tarpc::context;
use tokio::time;

#[derive(Clone)]
pub struct RelayerClient(SocketAddr);

impl RelayerClient {
    pub fn new_with_socket_addr(addr: SocketAddr) -> Self {
        RelayerClient(addr)
    }
}

#[tarpc::server]
impl PriceService for RelayerClient {
    async fn add_prices(self, _: context::Context, data: Vec<u8>, signature: Bytes) -> String {
        let sleep_time = Duration::from_millis(3000);
        time::sleep(sleep_time).await;

        let tokens = ethers::abi::decode(
            &[ParamType::Array(Box::new(ParamType::Tuple(vec![
                ParamType::Uint(256),
                ParamType::Uint(256),
                ParamType::String,
            ])))],
            &data,
        )
        .expect("failed to decode message");

        let tokens = tokens
            .first()
            .expect("decoded message must be non empty")
            .to_owned()
            .into_array()
            .expect("token must be able to be converted to array");

        let prices: Vec<Price> = tokens
            .clone()
            .into_iter()
            .map(|t| t.try_into().expect("unable to convert abi token to price"))
            .collect();

        let resp = format!(
            "Server received {:?} with signature length {}! You are connected from {}",
            prices,
            signature.len(),
            self.0
        );

        eprintln!("{resp}");

        resp
    }
}
