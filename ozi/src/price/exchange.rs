use std::str::FromStr;

use async_trait::async_trait;

use ethers::{
    abi::Token,
    signers::LocalWallet,
    types::{Bytes, H256},
};
use reqwest::Url;

use super::{
    types::{CollateralPair, Price},
    utils,
};

#[async_trait]
pub trait Exchange {
    fn url() -> Url;
    async fn fetch_prices(&self, collateral_pairs: Vec<CollateralPair>) -> Vec<Price>;

    async fn run(
        &self,
        collateral_pairs: Vec<CollateralPair>,
        private_key: String,
    ) -> (Vec<u8>, Bytes) {
        dotenv::dotenv().ok();
        let key = std::env::var(private_key).expect("Ethereum key must be set in your .env file");
        let wallet = LocalWallet::from_bytes(
            H256::from_str(key.as_str())
                .expect("Invalid hex private key")
                .as_bytes(),
        )
        .unwrap();

        let prices = self.fetch_prices(collateral_pairs).await;
        let tokens = Token::Array(prices.into_iter().map(|price| price.into()).collect());
        let encoded_message = ethers::abi::encode(&[tokens]);
        let signature = utils::sign_message(wallet, encoded_message.clone()).await;

        (encoded_message, signature)
    }
}