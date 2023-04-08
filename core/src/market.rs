use crate::asset::Asset;
use chrono::prelude::*;

/// Historical market data for a given pair.
pub struct Market {
    base_asset: Asset,  // => ETH <= / USD
    quote_asset: Asset, // ETH / => USD <=
    hloc_datas: Vec<Hloc>,
}

pub struct Hloc {
    open_datetime_utc: Option<DateTime<Utc>>,
    close_datetime_utc: Option<DateTime<Utc>>,
    high_price: Option<f64>,
    low_price: Option<f64>,
    open_price: Option<f64>,
    close_price: Option<f64>,
    volume: Option<f64>,
}

impl Market {
    pub fn new(base_asset: Asset, quote_asset: Asset, hloc_datas: Vec<Hloc>) -> Self {
        Self {
            base_asset,
            quote_asset,
            hloc_datas,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn market_new() {
        let market = Market::new(
            Asset::new(String::from("ETH"), String::from("Ether")),
            Asset::new(String::from("LUSD"), String::from("Liquity USD")),
            Vec::new(),
        );
        assert_eq!(market.hloc_datas.len(), 0);
    }
}
