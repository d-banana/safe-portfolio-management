use crate::asset::Asset;
use crate::position::Position;
use std::collections::HashMap;

pub struct Portfolio {
    pub balances: HashMap<Asset, f64>,
    pub positions: Vec<Position>,
}
