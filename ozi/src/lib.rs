mod exchanges;
mod network;
mod service;
mod utils;

pub use exchanges::*;
pub use service::*;

use ethers::types::Bytes;

#[tarpc::service]
pub trait PriceService {
    async fn add_prices(prices: Vec<u8>, signature: Bytes) -> String;
}
