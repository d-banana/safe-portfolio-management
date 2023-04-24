use thiserror::Error;

pub struct Tick {
    pub price: f64,
    pub time: i64,
    pub volume: f64,
    pub is_up: bool,
}

#[derive(Error, Debug, PartialEq)]
pub enum TickError {
    #[error("Price should be greater than 0({0})")]
    PriceShouldBeGtZero(f64),
    #[error("Volume should be greater than 0({0})")]
    VolumeShouldBeGtZero(f64),
}

impl Tick {
    pub fn new(price: f64, time: i64, volume: f64, is_up: bool) -> Result<Self, TickError> {
        let is_price_gt_zero = price > 0.0;
        if !is_price_gt_zero {
            return Err(TickError::PriceShouldBeGtZero(price));
        }
        let is_volume_gt_zero = volume > 0.0;
        if !is_volume_gt_zero {
            return Err(TickError::VolumeShouldBeGtZero(volume));
        }
        Ok(Self {
            price,
            time,
            volume,
            is_up,
        })
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum HlocError {
    #[error("Price should always be positive...")]
    PriceShouldPositiveError,
    #[error("Volume should always be positive...")]
    VolumeShouldPositiveError,
}
#[derive(Clone)]
pub struct Hloc {
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub close: f64,
    pub time: i64,
    pub volume: f64,
}

impl Hloc {
    pub fn new(
        high: f64,
        low: f64,
        open: f64,
        close: f64,
        time: i64,
        volume: f64,
    ) -> Result<Self, HlocError> {
        if high.is_sign_negative()
            || low.is_sign_negative()
            || open.is_sign_negative()
            || close.is_sign_negative()
        {
            return Err(HlocError::PriceShouldPositiveError);
        }
        if volume.is_sign_negative() {
            return Err(HlocError::VolumeShouldPositiveError);
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
    pub fn from_tick_vec(
        ticks: Vec<Tick>,
        duration_millisecond: i64,
    ) -> Result<Vec<Hloc>, HlocError> {
        let mut hlocs: Vec<Hloc> = Vec::new();
        if ticks.is_empty() {
            return Ok(hlocs);
        }
        let first_tick = ticks.first().unwrap();
        let mut slice = first_tick.time / duration_millisecond;
        let mut current_hloc = Hloc::new(
            first_tick.price,
            first_tick.price,
            first_tick.price,
            first_tick.price,
            first_tick.time - (first_tick.time % duration_millisecond),
            first_tick.volume,
        )?;

        for tick in &ticks {
            let current_slice = tick.time / duration_millisecond;
            let is_new_period = current_slice > slice;
            if is_new_period {
                slice = tick.time / duration_millisecond;
                hlocs.push(current_hloc.clone());
                current_hloc = Hloc::new(
                    tick.price,
                    tick.price,
                    current_hloc.close,
                    tick.price,
                    tick.time - (tick.time % duration_millisecond),
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

    #[test]
    fn tick_new() {
        let tick = Tick::new(1_000.0, 100, 10.0, true);
        assert!(tick.is_ok());
        let tick = tick.unwrap();
        assert_eq!(tick.price, 1_000.0);
        assert_eq!(tick.time, 100);
        assert_eq!(tick.volume, 10.0);
        assert!(tick.is_up);
    }
}
