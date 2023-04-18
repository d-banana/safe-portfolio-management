use rand::{thread_rng, Rng};
use thiserror::Error;

const BULLISH: f64 = 0.5;
const VOLUME: (f64, f64) = (0.01, 1.5);
const BIG_VOLUME_OCCURRENCE: f64 = 0.0001;
const BIG_VOLUME: (f64, f64) = (2.0, 100.0);

#[derive(Error, Debug, PartialEq)]
pub enum TakerError {
    #[error("Liquidity multiplier should always be positive...")]
    LiquidityMultiplierShouldBePositive,
    #[error("Fear should be between 0.0 and 1.0")]
    FearIsPercentDecimal,
    #[error("Greed should be between 0.0 and 1.0")]
    GreedIsPercentDecimal,
}
pub struct TakeLiquidity {
    pub is_buy: bool,
    pub quantity: f64,
}

impl TakeLiquidity {
    pub fn new(is_buy: bool, quantity: f64) -> Self {
        Self { is_buy, quantity }
    }
}

pub struct Taker {
    pub liquidity_multiplier: f64,
    pub fear: f64,
    pub greed: f64,
}

impl Taker {
    pub fn new(liquidity_multiplier: f64, fear: f64, greed: f64) -> Result<Self, TakerError> {
        if liquidity_multiplier.is_sign_negative() {
            return Err(TakerError::LiquidityMultiplierShouldBePositive);
        }
        let is_fear_pct_decimal = fear >= 0.0 && fear <= 1.0;
        if !is_fear_pct_decimal {
            return Err(TakerError::FearIsPercentDecimal);
        }
        let is_greed_pct_decimal = greed >= 0.0 && greed <= 1.0;
        if !is_greed_pct_decimal {
            return Err(TakerError::GreedIsPercentDecimal);
        }
        Ok(Self {
            liquidity_multiplier,
            fear,
            greed,
        })
    }

    pub fn take_liquidity(&self) -> Result<TakeLiquidity, TakerError> {
        let mut rng = thread_rng();
        let quantity = if rng.gen_bool(BIG_VOLUME_OCCURRENCE) {
            rng.gen_range(BIG_VOLUME.0..BIG_VOLUME.1)
        } else {
            rng.gen_range(VOLUME.0..VOLUME.1)
        };
        Ok(TakeLiquidity::new(
            rng.gen_bool(BULLISH + (self.greed - self.fear) / 2.0),
            quantity,
        ))
    }
    pub fn update_sentiment(&mut self, stochastic_slow: f64) -> Result<(), TakerError> {
        let is_overbought = stochastic_slow > 0.8;
        if is_overbought {}
        Ok(())
    }
}
