use crate::asset::Asset;
use crate::market::Market;
use crate::portfolio::Portfolio;
use crate::position::Position;
use crate::strategy::Strategy;
use chrono::{prelude::*, Duration};
use ethers::types::Address;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StrategyError {
    #[error("Not enough reserve asset {0} >= {1}")]
    ReserveAssetBalanceNotEnough(f64, f64),
    #[error("Need to wait {0} >= {1}")]
    NeedToWait(DateTime<Utc>, DateTime<Utc>),
}

pub struct DollarCostAveraging {
    open_datetime_utc: Option<DateTime<Utc>>,
    close_datetime_utc: Option<DateTime<Utc>>,
    reserve_asset: Asset,
    buy_asset: Asset,
    interval_duration: Duration,
    interval_reserve_quantity: f64,
}

impl DollarCostAveraging {
    pub fn new(
        open_datetime_utc: Option<DateTime<Utc>>,
        close_datetime_utc: Option<DateTime<Utc>>,
        reserve_asset: Asset,
        buy_asset: Asset,
        interval_duration: Duration,
        interval_reserve_quantity: f64,
    ) -> Self {
        Self {
            open_datetime_utc,
            close_datetime_utc,
            reserve_asset,
            buy_asset,
            interval_duration,
            interval_reserve_quantity,
        }
    }
}

impl Strategy for DollarCostAveraging {
    fn check_new_position(
        &self,
        portfolio: Portfolio,
        markets: Vec<Market>,
    ) -> Result<Vec<Position>, Box<dyn std::error::Error>> {
        let now = Utc::now();
        let positions = vec![Position::new(
            Some(now),
            None,
            self.buy_asset.clone(),
            self.interval_reserve_quantity,
            None,
        )];

        let reserve_balance = *portfolio.balances.get(&self.reserve_asset).unwrap_or(&0f64);
        let is_reserve_asset_enough = reserve_balance.ge(&self.interval_reserve_quantity);
        if is_reserve_asset_enough == false {
            return Err(Box::new(StrategyError::ReserveAssetBalanceNotEnough(
                reserve_balance,
                self.interval_reserve_quantity,
            )));
        }

        let is_first_position = portfolio.positions.is_empty();
        if is_first_position == true {
            return Ok(positions);
        }

        let next_position_datetime = portfolio
            .positions
            .last()
            .unwrap()
            .open_datetime_utc
            .unwrap_or(now)
            + self.interval_duration;
        let is_wait_done = now.ge(&next_position_datetime);
        if is_wait_done == true {
            return Ok(positions);
        }

        Err(Box::new(StrategyError::NeedToWait(
            now,
            next_position_datetime,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dollar_cost_averaging_new() {
        let reserve_asset = Asset::new(Address::random(), String::from("LUSD"), 8);
        let buy_asset = Asset::new(Address::random(), String::from("ETH"), 8);
        let interval_duration = Duration::days(7);
        let interval_reserve_quantity = 500f64;

        let dca = DollarCostAveraging::new(
            Some(Utc::now()),
            None,
            reserve_asset,
            buy_asset,
            interval_duration,
            interval_reserve_quantity,
        );
        assert_eq!(dca.interval_reserve_quantity, interval_reserve_quantity);
    }
}
