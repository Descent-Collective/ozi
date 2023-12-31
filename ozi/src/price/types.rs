use anyhow::{anyhow, Context, Result};
use ethers::{abi::Token, types::U256, utils::ParseUnits};
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use strum::{Display, EnumString};

#[derive(
    Debug,
    Serialize,
    Deserialize,
    EnumString,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Copy,
    Clone,
    Display,
    Hash,
)]
pub enum Symbol {
    USD,
    ETH,
    JPY,
    NGN,
    USDC,
}

impl Symbol {
    fn parse_symbols(string: &str, delimiter: &str) -> Result<(Symbol, Symbol)> {
        if let [Ok(aa), Ok(aaa)] = string
            .split(delimiter)
            .map(|a| a.parse::<Symbol>())
            .collect::<Vec<_>>()[..]
        {
            Ok((aa.clone(), aaa.clone()))
        } else {
            let error = anyhow!("Unable to parse symbols from string: {string}");
            Err(error)
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, PartialOrd, Eq, Ord)]
pub struct CollateralPair {
    pub fsym: Symbol,
    pub tsym: Symbol,
}

impl ToString for CollateralPair {
    fn to_string(&self) -> String {
        format!("{}/{}", self.fsym, self.tsym)
    }
}

impl CollateralPair {
    #[allow(dead_code)]
    fn as_64_len_hex_string(&self) -> String {
        let hexstr = hex::encode(self.to_string());
        let tstr = format!("{:0<64}", hexstr);
        println!("{} -> {}", hexstr, tstr);
        tstr
    }

    fn from_64_len_hex_string(string: impl ToString) -> CollateralPair {
        let tstr = string.to_string();
        // todo: there has to be a better way to do this
        let tstr = tstr.trim_end_matches('0');
        CollateralPair::try_from(tstr).expect("")
    }

    #[cfg(test)]
    fn parse_collateral_pairs(mut pairs: Vec<String>) -> Result<Vec<CollateralPair>> {
        pairs.sort();
        pairs
            .into_iter()
            .map(|p| CollateralPair::try_from(p.as_str()))
            .collect()
    }
}

impl TryFrom<&str> for CollateralPair {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match Symbol::parse_symbols(value, "/") {
            Ok((fsym, tsym)) => Ok(CollateralPair { fsym, tsym }),
            Err(e) => Err(anyhow!("Unable to parse collateral pair: {e:?}")),
        }
    }
}

impl<'de> Deserialize<'de> for CollateralPair {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        CollateralPair::try_from(s).map_err(D::Error::custom)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Price {
    pub fsym: Symbol,
    pub tsym: Symbol,
    pub rate: f32,
    pub timestamp: u64,
}

impl Price {
    const ETHERS_UNIT: u32 = 6;

    fn collateral_pair(&self) -> CollateralPair {
        CollateralPair {
            fsym: self.fsym,
            tsym: self.tsym,
        }
    }
}

impl Into<Token> for Price {
    fn into(self) -> Token {
        let parsed_price = ethers::utils::parse_units(self.rate, Price::ETHERS_UNIT)
            .expect("Failed to convert float to ethers unit");

        let price_as_u256 = match parsed_price {
            ParseUnits::U256(a) => a,
            ParseUnits::I256(_) => panic!("Negative values not allowed for exchange rate"),
        };
        // let collateral_pair_as_hex = self.collateral_pair().as_64_len_hex_string();

        Token::Tuple(vec![
            Token::Uint(price_as_u256),
            Token::Uint(
                U256::try_from(self.timestamp)
                    .expect("Could not convert from u64 timestamp to u256"),
            ),
            Token::String(self.collateral_pair().to_string()),
            // Token::FixedBytes(Vec::from(collateral_pair_as_hex)),
        ])
    }
}

impl TryFrom<Token> for Price {
    type Error = anyhow::Error;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.into_tuple() {
            Some(tuple) => {
                match (
                    tuple[0].clone().into_uint(),
                    tuple[1].clone().into_uint(),
                    tuple[2].clone().into_string(),
                ) {
                    (Some(price_as_u256), Some(timestamp), Some(collateral_pair)) => {
                        let rate = ethers::utils::format_units(price_as_u256, Price::ETHERS_UNIT)
                            .context("")?
                            .parse::<f32>()
                            .context("")?;
                        let timestamp = timestamp.as_u64();
                        let cp = CollateralPair::try_from(collateral_pair.as_str())?;

                        Ok(Price {
                            fsym: cp.fsym,
                            tsym: cp.tsym,
                            rate,
                            timestamp,
                        })
                    }
                    _ => Err(anyhow!("Unable to deconstruct token tuple")),
                }
            }
            _ => Err(anyhow!("Token must be convertible to tuple")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_symbols() {
        let pair = Symbol::parse_symbols("USD/JPY".into(), "/");
        assert!(pair.is_ok());
        assert_eq!((Symbol::USD, Symbol::JPY), pair.unwrap());

        let pair = Symbol::parse_symbols("ETH-JPY".into(), "-");
        assert!(pair.is_ok());
        let pair = pair.unwrap();
        assert_eq!((Symbol::ETH, Symbol::JPY), pair);
    }

    #[test]
    fn test_parse_collateral_pairs() {
        let collateral_pairs = CollateralPair::parse_collateral_pairs(vec![
            "USDC/NGN".into(),
            "JPY/USD".into(),
            "USDC/JPY".into(),
            "USDC/ETH".into(),
            "JPY/NGN".into(),
        ]);
        assert!(collateral_pairs.is_ok());
    }
}
