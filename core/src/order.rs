use crate::asset::Asset;

/// Sell a given quantity of asset A for the best price available of asset B.
#[derive(PartialEq, Debug, Clone)]
pub struct MarketOrder {
    pub asset_sell: Asset,
    pub asset_buy: Asset,
    pub quantity_sell: f64,
}

impl MarketOrder {
    pub fn new(asset_sell: Asset, asset_buy: Asset, quantity_sell: f64) -> Self {
        Self {
            asset_sell,
            asset_buy,
            quantity_sell,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn market_order_new() {
        let order = MarketOrder::new(
            Asset::new(String::from("LUSD"), String::from("Liquity USD")),
            Asset::new(String::from("ETH"), String::from("Ether")),
            1900f64,
        );
        assert_eq!(order.quantity_sell, 1900f64);
    }
}
