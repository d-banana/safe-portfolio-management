use crate::market::Hloc;
use crate::runner::Runner;
use chrono::{DateTime, Duration, TimeZone, Utc};
pub use core::*;

mod maker;
mod runner;
mod taker;

pub fn generate_price_graph() -> Vec<(DateTime<Utc>, f64)> {
    let mut runner = Runner::new(200 * 24 * 60 * 60 * 1000, 0.001, 1_900.0).unwrap();

    let ticks = runner.run().unwrap();
    let hlocs = Hloc::from_tick_vec(ticks, 4 * 60 * 60 * 1000).unwrap();
    let mut price_chart = Vec::new();
    for hloc in &hlocs {
        price_chart.push((
            DateTime::<Utc>::MIN_UTC + Duration::milliseconds(hloc.time),
            hloc.open,
        ));
    }
    price_chart
}
