use crate::maker::{Maker, MakerError, ProvideLiquidity};
use crate::market::{Tick, TickError};
use crate::taker::{TakeLiquidity, Taker, TakerError};
use chrono::{DateTime, Duration, Utc};
use rand::{thread_rng, Rng};
use thiserror::Error;

// RUNNER
const GRANULARITY_EPOCH_MILLISECOND: (u64, u64) = (5, 30_000);
const START_DATE: DateTime<Utc> = DateTime::<Utc>::MIN_UTC;
const MOVING_AVERAGE_DURATION_MILLISECOND: u64 = 24 * 60 * 60 * 100;
const LIQUIDITY_MULTIPLIER: (f64, f64) = (1.0, 4.0);

// TAKER
const FEAR_START: f64 = 0.5;
const GREED_START: f64 = 0.5;

#[derive(Error, Debug, PartialEq)]
pub enum RunnerError {
    #[error("Price increment should be between 0.0 and 1.0")]
    PriceIncrementIsPercentDecimal,
    #[error("Price should always be positive...")]
    PriceShouldPositiveError,
    #[error("Something went wrong with Maker(`{0}`)")]
    MakerError(MakerError),
    #[error("Something went wrong with Taker(`{0}`)")]
    TakerError(TakerError),
    #[error("Something went wrong with Tick(`{0}`)")]
    TickError(TickError),
    #[error("Unexpected error...")]
    UnexpectedError,
}

pub struct Runner {
    pub duration_millisecond: u64,
    pub price_increment_percent: f64,
    pub start_price: f64,
}

impl Runner {
    pub fn new(
        duration_millisecond: u64,
        price_increment_percent: f64,
        start_price: f64,
    ) -> Result<Self, RunnerError> {
        if start_price.is_sign_negative() {
            return Err(RunnerError::PriceShouldPositiveError);
        }
        let is_price_increment_pct_decimal =
            price_increment_percent >= 0.0 && price_increment_percent <= 1.0;
        if !is_price_increment_pct_decimal {
            return Err(RunnerError::PriceIncrementIsPercentDecimal);
        }

        Ok(Self {
            duration_millisecond,
            price_increment_percent,
            start_price,
        })
    }

    pub fn run(&self) -> Result<Vec<Tick>, RunnerError> {
        let mut rng = thread_rng();
        let mut ticks: Vec<Tick> = Vec::new();
        let mut maker: Maker = self.init_maker()?;
        let mut taker: Taker = self.init_taker()?;
        let mut last_price = self.start_price;
        let mut moving_average: Option<f64> = None;

        let mut current_time_millisecond: u64 = 0;
        while current_time_millisecond < self.duration_millisecond {
            let (liquidity_ask, liquidity_bid) = maker
                .provide_liquidity()
                .map_err(|e| RunnerError::MakerError(e))?;
            let take_liquidity = taker
                .take_liquidity()
                .map_err(|e| RunnerError::TakerError(e))?;

            if take_liquidity.is_buy {
                ticks.append(&mut self.match_order(
                    last_price,
                    current_time_millisecond,
                    liquidity_ask,
                    take_liquidity,
                )?);
            } else {
                ticks.append(&mut self.match_order(
                    last_price,
                    current_time_millisecond,
                    liquidity_bid,
                    take_liquidity,
                )?);
            }

            if ticks.last().is_some() {
                last_price = ticks.last().ok_or(RunnerError::UnexpectedError)?.price;
            }
            current_time_millisecond +=
                rng.gen_range(GRANULARITY_EPOCH_MILLISECOND.0..GRANULARITY_EPOCH_MILLISECOND.1);
        }

        Ok(ticks)
    }

    fn init_maker(&self) -> Result<Maker, RunnerError> {
        let maker = Maker::new(LIQUIDITY_MULTIPLIER.0);
        maker.map_err(|e| RunnerError::MakerError(e))
    }

    fn init_taker(&self) -> Result<Taker, RunnerError> {
        let taker = Taker::new(LIQUIDITY_MULTIPLIER.0, FEAR_START, GREED_START);
        taker.map_err(|e| RunnerError::TakerError(e))
    }

    fn match_order(
        &self,
        last_price: f64,
        current_time_millisecond: u64,
        provide_liquidity: ProvideLiquidity,
        take_liquidity: TakeLiquidity,
    ) -> Result<Vec<Tick>, RunnerError> {
        let mut rng = thread_rng();
        let mut ticks: Vec<Tick> = Vec::new();
        let mut volume_left = take_liquidity.quantity;
        let mut liquidity_tick = provide_liquidity.liquidity_by_tick;
        let mut price = if take_liquidity.is_buy {
            last_price
                + (last_price * self.price_increment_percent * provide_liquidity.spread_tick as f64)
        } else {
            last_price
                - (last_price * self.price_increment_percent * provide_liquidity.spread_tick as f64)
        };
        let mut current_time_millisecond = current_time_millisecond as i64;

        while volume_left > 0.0 && liquidity_tick > 0.0 {
            let volume = if volume_left < liquidity_tick {
                volume_left
            } else {
                liquidity_tick
            };

            ticks.push(
                Tick::new(
                    price,
                    current_time_millisecond,
                    volume,
                    take_liquidity.is_buy,
                )
                .map_err(|e| RunnerError::TickError(e))?,
            );

            volume_left -= volume;
            liquidity_tick += provide_liquidity.liquidity_rate_of_change_by_tick;
            price = if take_liquidity.is_buy {
                price + (price * self.price_increment_percent)
            } else {
                price - (price * self.price_increment_percent)
            };
        }
        Ok(ticks)
    }
}
