use crate::asset::Asset;
use crate::order::MarketOrder;
use chrono::{prelude::*, Duration};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DollarCostAveragingError {
    #[error("Not enough asset to sell {0} >= {1}")]
    SellAssetBalanceNotEnough(f64, f64),
    #[error("Need to wait {0} >= {1}")]
    NeedToWait(DateTime<Utc>, DateTime<Utc>),
}

/// Periodically sell the same amount of an asset A to buy an asset B.
pub struct DollarCostAveraging {
    sell_asset: Asset,
    buy_asset: Asset,
    interval_duration: Duration,
    interval_sell_quantity: f64,
}

impl DollarCostAveraging {
    pub fn new(
        sell_asset: Asset,
        buy_asset: Asset,
        interval_duration: Duration,
        interval_sell_quantity: f64,
    ) -> Self {
        Self {
            sell_asset,
            buy_asset,
            interval_duration,
            interval_sell_quantity,
        }
    }

    pub fn check_new_order(
        &self,
        last_position_datetime: &Option<DateTime<Utc>>,
        sell_balance: f64,
    ) -> Result<Option<MarketOrder>, DollarCostAveragingError> {
        let now = Utc::now();
        let order = MarketOrder::new(
            self.sell_asset.clone(),
            self.buy_asset.clone(),
            self.interval_sell_quantity,
        );

        let is_reserve_asset_enough = sell_balance.ge(&self.interval_sell_quantity);
        if !is_reserve_asset_enough {
            return Err(DollarCostAveragingError::SellAssetBalanceNotEnough(
                sell_balance,
                self.interval_sell_quantity,
            ));
        }

        let is_first_position = last_position_datetime.is_none();
        if is_first_position {
            return Ok(Some(order));
        }

        let next_position_datetime = last_position_datetime.unwrap() + self.interval_duration;
        let is_wait_done = now.ge(&next_position_datetime);
        if is_wait_done {
            return Ok(Some(order));
        }

        Err(DollarCostAveragingError::NeedToWait(
            now,
            next_position_datetime,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _dollar_cost_averaging_new() -> DollarCostAveraging {
        let sell_asset = Asset::new(String::from("LUSD"), String::from("Liquity USD"));
        let buy_asset = Asset::new(String::from("ETH"), String::from("Ether"));
        let interval_duration = Duration::days(7);
        let interval_sell_quantity = 500f64;

        let dca = DollarCostAveraging::new(
            sell_asset,
            buy_asset,
            interval_duration,
            interval_sell_quantity,
        );
        dca
    }

    #[test]
    fn dollar_cost_averaging_new() {
        let dca = _dollar_cost_averaging_new();
        assert_eq!(dca.interval_sell_quantity, 500f64);
    }

    #[test]
    fn dollar_cost_averaging_check_new_order_first() {
        let dca = _dollar_cost_averaging_new();
        let result = dca.check_new_order(&None, 1000f64);

        assert!(result.is_ok());
        let order = result.unwrap();
        assert!(order.is_some());
        let order = order.unwrap();

        assert_eq!(order.asset_sell, dca.sell_asset);
        assert_eq!(order.asset_buy, dca.buy_asset);
        assert_eq!(order.quantity_sell, dca.interval_sell_quantity);
    }

    #[test]
    fn dollar_cost_averaging_check_new_order_success() {
        let dca = _dollar_cost_averaging_new();
        let result = dca.check_new_order(&Some(Utc::now() - Duration::days(8)), 1000f64);

        assert!(result.is_ok());
        let order = result.unwrap();
        assert!(order.is_some());
        let order = order.unwrap();

        assert_eq!(order.asset_sell, dca.sell_asset);
        assert_eq!(order.asset_buy, dca.buy_asset);
        assert_eq!(order.quantity_sell, dca.interval_sell_quantity);
    }

    #[test]
    fn dollar_cost_averaging_check_new_order_wait() {
        let dca = _dollar_cost_averaging_new();
        let result = dca.check_new_order(
            &Some(Utc::now() - Duration::days(6) - Duration::hours(23) - Duration::minutes(59)),
            1000f64,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Need to wait"));
    }

    #[test]
    fn dollar_cost_averaging_check_new_order_not_enough() {
        let dca = _dollar_cost_averaging_new();
        let result = dca.check_new_order(&Some(Utc::now() - Duration::days(8)), 499f64);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            DollarCostAveragingError::SellAssetBalanceNotEnough(499f64, 500f64)
        );
    }
}
