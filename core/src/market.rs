use crate::asset::Asset;
use chrono::prelude::*;

pub struct Market {
    asset: Asset,
    hloc_datas: Vec<Hloc>,
}

struct Hloc {
    open_datetime_utc: Option<DateTime<Utc>>,
    close_datetime_utc: Option<DateTime<Utc>>,
    high_price: Option<f64>,
    low_price: Option<f64>,
    open_price: Option<f64>,
    close_price: Option<f64>,
    volume: Option<f64>,
}
