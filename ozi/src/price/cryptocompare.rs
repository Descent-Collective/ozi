use std::collections::HashMap;

use async_trait::async_trait;
use futures::{stream, StreamExt};
use reqwest::{Client, RequestBuilder, Url};
use serde::Deserialize;

use super::{
    exchange::Exchange,
    types::{CollateralPair, Price, Symbol},
    utils,
};

pub struct CryptoCompare {
    client: Client,
}

impl CryptoCompare {
    pub fn new() -> CryptoCompare {
        CryptoCompare {
            client: reqwest::Client::new(),
        }
    }

    fn gen_queries(
        &self,
        mut collateral_pairs: Vec<CollateralPair>,
    ) -> Vec<(Symbol, RequestBuilder)> {
        collateral_pairs.sort();
        let mut fsyms = collateral_pairs.iter().map(|f| f.fsym).collect::<Vec<_>>();
        fsyms.dedup();

        let queries = fsyms
            .iter()
            .map(|fsym| {
                let mut tsyms = collateral_pairs
                    .clone()
                    .iter()
                    .filter_map(move |pair| {
                        if pair.fsym.eq(&fsym) {
                            Some(pair.tsym.to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                tsyms.sort();

                let query = self.client.get(CryptoCompare::url());
                (
                    fsym.to_owned(),
                    query.query(&[("fsym", fsym.to_string()), ("tsyms", tsyms.join(","))]),
                )
            })
            .collect::<Vec<_>>();

        queries
    }
}

#[derive(Debug, Deserialize)]
struct Response {
    #[serde(flatten)]
    inner: HashMap<Symbol, f32>,
}

impl Into<HashMap<Symbol, f32>> for Response {
    fn into(self) -> HashMap<Symbol, f32> {
        self.inner.into_iter().map(|x| (x.0, x.1)).collect()
    }
}

#[async_trait]
impl Exchange for CryptoCompare {
    fn url() -> Url {
        Url::try_from("https://min-api.cryptocompare.com/data/price").unwrap()
    }

    async fn fetch_prices(&self, collateral_pairs: Vec<CollateralPair>) -> Vec<Price> {
        let fsym_to_rbs = self.gen_queries(collateral_pairs);
        let concurrent_reqs = fsym_to_rbs.len();

        let bodies = stream::iter(fsym_to_rbs)
            .map(|(fsym, rb)| async move {
                let resp = rb.send().await?.json::<Response>().await;

                resp.map(|data| {
                    data.inner
                        .into_iter()
                        .map(|(tsym, rate)| Price {
                            fsym,
                            tsym,
                            rate,
                            timestamp: utils::get_unix_timestamp(),
                        })
                        .collect::<Vec<_>>()
                })
            })
            .buffer_unordered(concurrent_reqs);

        let results = bodies
            .filter_map(|body| async move {
                match body {
                    Ok(_) => body.ok(),
                    Err(err) => {
                        println!("wahala {err:?}");
                        None
                    }
                }
            })
            .collect::<Vec<_>>()
            .await;

        results.into_iter().flatten().collect()
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_gen_query() {
        let r = CryptoCompare::new().gen_queries(vec![
            CollateralPair {
                fsym: Symbol::JPY,
                tsym: Symbol::NGN,
            },
            CollateralPair {
                fsym: Symbol::USDC,
                tsym: Symbol::JPY,
            },
            CollateralPair {
                fsym: Symbol::JPY,
                tsym: Symbol::USD,
            },
            CollateralPair {
                fsym: Symbol::USDC,
                tsym: Symbol::ETH,
            },
            CollateralPair {
                fsym: Symbol::USDC,
                tsym: Symbol::NGN,
            },
        ]);

        assert_eq!(r.len(), 2);

        let rbs = r
            .into_iter()
            .filter_map(|(_fsym, rb)| rb.build().ok())
            .collect::<Vec<_>>();

        assert_eq!(rbs[0].url().query(), Some("fsym=JPY&tsyms=NGN%2CUSD"));
        assert_eq!(
            rbs[1].url().query(),
            Some("fsym=USDC&tsyms=ETH%2CJPY%2CNGN")
        );
    }
}
