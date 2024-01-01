use async_trait::async_trait;
use reqwest::{Client, Url};

use super::{CollateralPair, Exchange, Price};

pub struct NumeroFx {
    #[allow(dead_code)]
    client: Client,
}

impl NumeroFx {
    pub fn new() -> NumeroFx {
        NumeroFx {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Exchange for NumeroFx {
    fn url() -> Url {
        Url::try_from("https://api.numerofx.com/api/v1/developer/rates").unwrap()
    }

    async fn fetch_prices(&self, _collateral_pairs: Vec<CollateralPair>) -> Vec<Price> {
        todo!()
    }
}
