use crate::market::Hloc;
use crate::runner::Runner;
use chrono::{DateTime, Duration, Utc};
pub use core::*;

mod actor;
mod runner;

pub fn generate_price_graph() -> Vec<(DateTime<Utc>, f64)> {
    let mut runner = Runner::default();

    let ticks = runner.run(0, 200 * 24 * 60 * 60 * 1_000, 1_000.0).unwrap();
    let hlocs = Hloc::from_tick_vec(ticks, 4 * 60 * 60 * 1_000).unwrap();
    let mut price_chart = Vec::new();
    for hloc in &hlocs {
        price_chart.push((
            DateTime::<Utc>::MIN_UTC + Duration::milliseconds(hloc.time),
            hloc.open,
        ));
    }
    price_chart
}
