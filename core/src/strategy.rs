use crate::market::Market;
use crate::portfolio::Portfolio;
use crate::position::Position;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StrategyError {
    #[error("Failed to check new position({0})")]
    FailedToCheckNewPosition(String),
}

trait Strategy {
    fn check_new_position(
        &self,
        portfolio: Portfolio,
        markets: [&Market],
    ) -> Result<&[Position], StrategyError>;
}
