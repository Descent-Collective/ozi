use ethers::types::Bytes;

#[tarpc::service]
pub trait PriceService {
    async fn add_prices(prices: Vec<u8>, signature: Bytes) -> String;
}
