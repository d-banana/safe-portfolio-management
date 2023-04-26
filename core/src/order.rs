use crate::asset::Asset;
use ethers::types::U64;

/// Sell a given quantity of asset A for the best price available of asset B.
#[derive(PartialEq, Debug, Clone)]
pub struct MarketOrder {
    pub asset_sell: Asset,
    pub asset_buy: Asset,
    pub quantity_sell: U64,
}

impl MarketOrder {
    pub fn new(asset_sell: Asset, asset_buy: Asset, quantity_sell: U64) -> Self {
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
            U64::from(1_900) * U64::exp10(6),
        );
        assert_eq!(order.asset_sell.id, "LUSD");
        assert_eq!(order.asset_buy.id, "ETH");
        assert_eq!(order.quantity_sell, U64::from(1_900) * U64::exp10(6));
    }
}
