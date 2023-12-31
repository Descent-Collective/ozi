use std::{net::SocketAddr, time::Duration};

use ethers::{abi::ParamType, types::Bytes};
use ozi::{Price, PriceService};
use tarpc::context;
use tokio::time;

#[derive(Clone)]
pub struct RelayerClient(pub SocketAddr);

#[tarpc::server]
impl PriceService for RelayerClient {
    async fn add_prices(
        self,
        _: context::Context,
        encoded_message: Vec<u8>,
        signature: Bytes,
    ) -> String {
        let sleep_time = Duration::from_millis(3000);
        time::sleep(sleep_time).await;

        let tokens = ethers::abi::decode(
            &[ParamType::Array(Box::new(ParamType::Tuple(vec![
                ParamType::Uint(256),
                ParamType::Uint(256),
                ParamType::String,
            ])))],
            &encoded_message,
        )
        .expect("failed to decode message");

        let tokens = tokens
            .first()
            .expect("decoded vec must be non empty")
            .to_owned()
            .into_array()
            .expect("token cannot be converted to array");

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

        println!("{resp}");

        resp
    }
}
