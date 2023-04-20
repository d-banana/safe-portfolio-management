use crate::market::Hloc;
use crate::runner::Runner;
use chrono::{DateTime, Duration, TimeZone, Utc};
pub use core::*;

mod market_state;
mod runner;

pub fn generate_price_graph() -> Vec<(DateTime<Utc>, f64)> {
    let mut runner = Runner::new(
        200 * 24 * 60 * 60 * 1000,
        0.1,
        1_900.0,
        (15, 15_000),
        (14 * 24 * 60 * 60 * 1000, 60 * 24 * 60 * 60 * 1000),
    );

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
