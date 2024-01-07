use std::str::FromStr;

use anyhow::{Context, Result};
use async_trait::async_trait;
use ethers::abi::Token;
use ethers::signers::LocalWallet;
use ethers::types::{Bytes, H256};
use reqwest::Url;

use super::utils;
use super::{CollateralPair, Price};

#[async_trait]
pub trait Exchange {
    fn name() -> &'static str;
    fn url() -> Url;
    async fn fetch_prices(&self, collateral_pairs: Vec<CollateralPair>) -> Vec<Price>;

    async fn run(
        &self,
        collateral_pairs: Vec<CollateralPair>,
        private_key: String,
    ) -> Result<(Vec<u8>, Bytes)> {
        dotenv::dotenv().ok();
        let key =
            std::env::var(private_key).context("Ethereum key must be set in your .env file")?;
        let wallet = LocalWallet::from_bytes(
            H256::from_str(key.as_str())
                .context("Invalid hex private key")?
                .as_bytes(),
        )
        .unwrap();

        let prices = self.fetch_prices(collateral_pairs).await;
        let tokens = Token::Array(prices.into_iter().map(|price| price.into()).collect());
        let encoded_message = ethers::abi::encode(&[tokens]);
        let signature = utils::sign_message(wallet, encoded_message.clone()).await;

        Ok((encoded_message, signature))
    }
}
