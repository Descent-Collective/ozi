use ethers::types::Bytes;

#[tarpc::service]
/// PriceService defines the available functions and
/// data types expected on ozi's rpc price service
pub trait PriceService {
    /// Adds an array of prices verifiable by the sender's signature
    async fn add_prices(data: Vec<u8>, signature: Bytes) -> String;
}
