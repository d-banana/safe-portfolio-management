use crate::Asset;
use chrono::{prelude::*, Duration};

pub struct DollarCostAveraging {
    open_datetime_utc: Option<DateTime<Utc>>,
    close_datetime_utc: Option<DateTime<Utc>>,
    reserve_asset: Asset,
    buy_asset: Asset,
    interval_duration: Duration,
    interval_reserve_quantity: f64,
}
