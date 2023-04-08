use crate::asset::Asset;
use chrono::prelude::*;
use ethers::types::{Address, TxHash};

/// A position to sell a given quantity of an asset A for a given minimum quantity of an asset B.
#[derive(PartialEq, Debug, Clone)]
pub struct Position {
    pub asset_sell: Asset,
    pub asset_buy: Asset,
    pub quantity_sell: f64,
    pub quantity_min_buy: f64,
}

impl Position {
    pub fn new(
        asset_sell: Asset,
        asset_buy: Asset,
        quantity_sell: f64,
        quantity_min_buy: f64,
    ) -> Self {
        Self {
            asset_sell,
            asset_buy,
            quantity_sell,
            quantity_min_buy,
        }
    }
}

/*
/// State of the position
/// INTERNAL = not sent to Safe
/// SUBMITTED = Waiting to be sign and executed on-chain
/// EXECUTED = Successfully executed on-chain
/// REFUSED = Safe owner refused the transaction
/// FAILED = Transaction sent on-chain but revert.
#[derive(PartialEq, Debug, Clone)]
pub enum PositionState {
    INTERNAL,
    SUBMITTED,
    EXECUTED,
    REFUSED,
    FAILED,
}

/// A future/open/closed position of an asset with a defined quantity.
#[derive(PartialEq, Debug, Clone)]
pub struct Position {
    pub open_datetime_utc: Option<DateTime<Utc>>,
    pub asset_sell: Asset,
    pub asset_buy: Asset,
    pub quantity_sell: f64,
    pub quantity_buy: Option<f64>,
    pub transaction_hash: Option<TxHash>,
    pub state: PositionState,
}

impl Position {
    pub fn new(
        open_datetime_utc: Option<DateTime<Utc>>,
        asset_sell: Asset,
        asset_buy: Asset,
        quantity_sell: f64,
        quantity_buy: Option<f64>,
        transaction_hash: Option<TxHash>,
        state: PositionState,
    ) -> Self {
        Self {
            open_datetime_utc,
            asset_sell,
            asset_buy,
            quantity_sell,
            quantity_buy,
            transaction_hash,
            state,
        }
    }
}*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_new() {
        let position = Position::new(
            Asset::new(String::from("LUSD"), String::from("Liquity USD")),
            Asset::new(String::from("ETH"), String::from("Ether")),
            1900f64,
            1f64,
        );
        assert_eq!(position.quantity_sell, 1900f64);
    }
}
