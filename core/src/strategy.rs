use crate::market::Market;
use crate::portfolio::Portfolio;
use crate::position::Position;

pub trait Strategy {
    fn check_new_position(
        &self,
        portfolio: Portfolio,
        markets: Vec<Market>,
    ) -> Result<Vec<Position>, Box<dyn std::error::Error>>;
}
