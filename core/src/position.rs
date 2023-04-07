use crate::asset::Asset;
use chrono::prelude::*;
use ethers::types::{Address, TxHash};

/// A future/open/closed position of an asset with a defined quantity.
pub struct Position {
    pub open_datetime_utc: Option<DateTime<Utc>>,
    pub close_datetime_utc: Option<DateTime<Utc>>,
    pub asset: Asset,
    pub quantity: f64,
    pub transaction_hash: Option<TxHash>,
}

impl Position {
    pub fn new(
        open_datetime_utc: Option<DateTime<Utc>>,
        close_datetime_utc: Option<DateTime<Utc>>,
        asset: Asset,
        quantity: f64,
        transaction_hash: Option<TxHash>,
    ) -> Self {
        Self {
            open_datetime_utc: open_datetime_utc,
            close_datetime_utc: close_datetime_utc,
            asset,
            quantity,
            transaction_hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_new() {
        let position = Position::new(
            Some(Utc::now()),
            None,
            Asset::new(Address::random(), String::from("ETH"), 8),
            512f64,
            None,
        );
        assert_eq!(position.quantity, 512f64);
    }
}
