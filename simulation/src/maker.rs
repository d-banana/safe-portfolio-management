use rand::{thread_rng, Rng};
use thiserror::Error;

const SPREAD: (usize, usize) = (1, 10);
const LIQUIDITY_BEST: (f64, f64) = (0.5, 2.0);
const LIQUIDITY_DEEP: (f64, f64) = (0.0, 3.0);

#[derive(Error, Debug, PartialEq)]
pub enum MakerError {
    #[error("Liquidity multiplier should always be positive...")]
    LiquidityMultiplierShouldBePositive,
}
pub struct ProvideLiquidity {
    pub spread_tick: usize,
    pub liquidity_by_tick: f64,
    pub liquidity_rate_of_change_by_tick: f64,
}

impl ProvideLiquidity {
    pub fn new(
        spread_tick: usize,
        liquidity_by_tick: f64,
        liquidity_rate_of_change_by_tick: f64,
    ) -> Self {
        Self {
            spread_tick,
            liquidity_by_tick,
            liquidity_rate_of_change_by_tick,
        }
    }
}

pub type Ask = ProvideLiquidity;
pub type Bid = ProvideLiquidity;
pub struct Maker {
    pub liquidity_multiplier: f64,
}

impl Maker {
    pub fn new(liquidity_multiplier: f64) -> Result<Self, MakerError> {
        if liquidity_multiplier.is_sign_negative() {
            return Err(MakerError::LiquidityMultiplierShouldBePositive);
        }
        Ok(Self {
            liquidity_multiplier,
        })
    }

    pub fn provide_liquidity(&self) -> Result<(Ask, Bid), MakerError> {
        let mut rng = thread_rng();
        let liquidity_ask = ProvideLiquidity::new(
            1,
            rng.gen_range(LIQUIDITY_BEST.0..LIQUIDITY_BEST.1),
            rng.gen_range(LIQUIDITY_DEEP.0..LIQUIDITY_DEEP.1),
        );
        let liquidity_bid = ProvideLiquidity::new(
            1,
            rng.gen_range(LIQUIDITY_BEST.0..LIQUIDITY_BEST.1),
            rng.gen_range(LIQUIDITY_DEEP.0..LIQUIDITY_DEEP.1),
        );

        Ok((liquidity_ask, liquidity_bid))
    }
}
