use ethers::types::U64;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Tick {
    pub price: U64,
    pub time: u64,
    pub volume: U64,
    pub is_up: bool,
    pub moving_average: Option<U64>,
    pub variance: Option<U64>,
}

#[derive(Error, Debug, PartialEq)]
pub enum TickError {
    #[error("Price should be greater than 0({0})")]
    PriceShouldBeGtZero(U64),
    #[error("Volume should be greater than 0({0})")]
    VolumeShouldBeGtZero(U64),
}

impl Tick {
    pub fn new(
        price: U64,
        time: u64,
        volume: U64,
        is_up: bool,
        moving_average: Option<U64>,
        variance: Option<U64>,
    ) -> Result<Self, TickError> {
        if price.is_zero() {
            return Err(TickError::PriceShouldBeGtZero(price));
        }
        if volume.is_zero() {
            return Err(TickError::VolumeShouldBeGtZero(volume));
        }
        Ok(Self {
            price,
            time,
            volume,
            is_up,
            moving_average,
            variance,
        })
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum HlocError {
    #[error("Price should be greater than 0(h:{high}, l:{low}, o:{open}, c:{close}, )")]
    PriceShouldBeGtZero {
        high: U64,
        low: U64,
        open: U64,
        close: U64,
    },
    #[error("Duration for HLOC should be greater than 0({0})")]
    DurationShouldBeGtZero(u64),
}
#[derive(Clone)]
pub struct Hloc {
    pub high: U64,
    pub low: U64,
    pub open: U64,
    pub close: U64,
    pub time: u64,
    pub volume: U64,
}

impl Hloc {
    pub fn new(
        high: U64,
        low: U64,
        open: U64,
        close: U64,
        time: u64,
        volume: U64,
    ) -> Result<Self, HlocError> {
        if high.is_zero() || low.is_zero() || open.is_zero() || close.is_zero() {
            return Err(HlocError::PriceShouldBeGtZero {
                high,
                low,
                open,
                close,
            });
        }
        Ok(Self {
            high,
            low,
            open,
            close,
            time,
            volume,
        })
    }
    pub fn from_tick_vec(ticks: Vec<Tick>, duration_ms: u64) -> Result<Vec<Hloc>, HlocError> {
        let is_duration_ms_gt_zero = duration_ms > 0;
        if !is_duration_ms_gt_zero {
            return Err(HlocError::DurationShouldBeGtZero(duration_ms));
        }

        let mut hlocs: Vec<Hloc> = Vec::new();
        if ticks.is_empty() {
            return Ok(hlocs);
        }
        let first_tick = ticks.first().unwrap();
        let mut slice = first_tick.time / duration_ms;
        let mut current_hloc = Hloc::new(
            first_tick.price,
            first_tick.price,
            first_tick.price,
            first_tick.price,
            first_tick.time - (first_tick.time % duration_ms),
            first_tick.volume,
        )?;

        for tick in &ticks {
            let current_slice = tick.time / duration_ms;
            let is_new_period = current_slice > slice;
            if is_new_period {
                slice = tick.time / duration_ms;
                hlocs.push(current_hloc.clone());
                current_hloc = Hloc::new(
                    tick.price,
                    tick.price,
                    current_hloc.close,
                    tick.price,
                    tick.time - (tick.time % duration_ms),
                    tick.volume,
                )?;
            }

            current_hloc.close = tick.price;
            current_hloc.volume += tick.volume;
            current_hloc.high = if current_hloc.high < tick.price {
                tick.price
            } else {
                current_hloc.high
            };
            current_hloc.low = if current_hloc.low > tick.price {
                tick.price
            } else {
                current_hloc.low
            };
        }

        Ok(hlocs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::types::U64;

    #[test]
    fn tick_new() {
        let tick = Tick::new(
            U64::from(1_000) * U64::exp10(6),
            100,
            U64::from(10) * U64::exp10(6),
            true,
            Some(U64::from(800) * U64::exp10(6)),
            Some(U64::from(50) * U64::exp10(6)),
        );
        assert!(tick.is_ok());
        let tick = tick.unwrap();
        assert_eq!(tick.price, U64::from(1_000) * U64::exp10(6));
        assert_eq!(tick.time, 100);
        assert_eq!(tick.volume, U64::from(10) * U64::exp10(6));
        assert!(tick.is_up);
        assert_eq!(tick.moving_average, Some(U64::from(800) * U64::exp10(6)));
        assert_eq!(tick.variance, Some(U64::from(50) * U64::exp10(6)));
    }

    #[test]
    fn hloc_new() {
        let hloc = Hloc::new(
            U64::from(1_100) * U64::exp10(6),
            U64::from(900) * U64::exp10(6),
            U64::from(1_000) * U64::exp10(6),
            U64::from(1_050) * U64::exp10(6),
            500,
            U64::from(12) * U64::exp10(6),
        );
        assert!(hloc.is_ok());
        let hloc = hloc.unwrap();
        assert_eq!(hloc.high, U64::from(1_100) * U64::exp10(6));
        assert_eq!(hloc.low, U64::from(900) * U64::exp10(6));
        assert_eq!(hloc.open, U64::from(1_000) * U64::exp10(6));
        assert_eq!(hloc.close, U64::from(1_050) * U64::exp10(6));
        assert_eq!(hloc.time, 500);
        assert_eq!(hloc.volume, U64::from(12) * U64::exp10(6));
    }
}
