use crate::asset::Asset;
use chrono::prelude::*;
use ethers::types::Address;

/// A future/open/closed position of an asset with a defined quantity.
pub struct Position {
    open_datetime_utc: Option<DateTime<Utc>>,
    close_datetime_utc: Option<DateTime<Utc>>,
    asset: Asset,
    quantity: f64,
}

impl Position {
    pub fn new(
        open_datetime_utc: Option<DateTime<Utc>>,
        close_datetime_utc: Option<DateTime<Utc>>,
        asset: Asset,
        quantity: f64,
    ) -> Self {
        Self {
            open_datetime_utc: open_datetime_utc,
            close_datetime_utc: close_datetime_utc,
            asset,
            quantity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_constructor() {
        let position = Position::new(
            Some(Utc::now()),
            None,
            Asset::new(Address::random(), String::from("ETH"), 8),
            512f64,
        );
        assert_eq!(position.quantity, 512f64);
    }
}
