use {ozi::CollateralPair, serde::Deserialize};

#[derive(Deserialize, Debug)]
pub struct NodeConfig {
    pub topic: String,
    /// Ethereum key to use for signing price messages.
    pub private_key: String,
    /// Specifies the interval in milliseconds between sending price messages.
    pub interval: u64,
    /// List of pairs to send price messages for
    /// This is the collateral we want to fetch prices for
    pub collateral_pairs: Vec<CollateralPair>,
    /// Port to listen on
    pub port: Option<String>,
    /// Bootstrap nodes for peer discovery with kademlia protocol
    pub bootstrap_nodes: Vec<String>,
    /// Relayer client address
    pub relayer_address: String,
}
