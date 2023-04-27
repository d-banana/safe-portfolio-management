use crate::market::Hloc;
use crate::runner::Runner;
use chrono::{DateTime, Duration, Utc};
pub use core::*;
use ethers::types::U64;
use std::ops::Div;

mod actor;
mod runner;

pub fn generate_price_graph() -> (Vec<(DateTime<Utc>, f64)>, Vec<(DateTime<Utc>, f64)>) {
    let mut runner = Runner::default();

    let ticks = runner
        .run(
            0,
            200 * 24 * 60 * 60 * 1_000,
            U64::from(1_000) * U64::exp10(6),
        )
        .unwrap();
    let is_hloc = false;
    if is_hloc {
        let hlocs = Hloc::from_tick_vec(ticks, 4 * 60 * 60 * 1_000).unwrap();
        let mut price_chart = (Vec::new(), Vec::new());
        for hloc in &hlocs {
            price_chart.0.push((
                DateTime::<Utc>::MIN_UTC + Duration::milliseconds(hloc.time as i64),
                hloc.open.div(U64::exp10(5)).as_u64() as f64 / 10.0,
            ));
            price_chart.1.push((
                DateTime::<Utc>::MIN_UTC + Duration::milliseconds(hloc.time as i64),
                hloc.open.div(U64::exp10(5)).as_u64() as f64 / 10.0,
            ));
        }
        return price_chart;
    }
    (
        ticks
            .iter()
            .map(|t| {
                (
                    DateTime::<Utc>::MIN_UTC + Duration::milliseconds(t.time as i64),
                    t.price.div(U64::exp10(5)).as_u64() as f64 / 10.0,
                )
            })
            .collect(),
        ticks
            .iter()
            .map(|t| {
                (
                    DateTime::<Utc>::MIN_UTC + Duration::milliseconds(t.time as i64),
                    t.moving_average
                        .unwrap_or(t.price)
                        .div(U64::exp10(5))
                        .as_u64() as f64
                        / 10.0,
                )
            })
            .collect(),
    )
}
