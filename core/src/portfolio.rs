use crate::asset::Asset;
use crate::position::Position;
use ethers::types::Address;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum PortfolioError {
    #[error("Not enough asset to sell {0} >= {1}")]
    NotEnoughAssetSell(f64, f64),
    #[error("Buy asset quantity not set")]
    BuyAssetQuantityNotSet(),
}

pub struct Portfolio {
    pub balances: HashMap<Asset, f64>,
    pub positions: Vec<Position>,
}
/*
impl Portfolio {
    pub fn new(balances: HashMap<Asset, f64>, positions: Vec<Position>) -> Self {
        Self {
            balances,
            positions,
        }
    }

    pub fn add_position(&mut self, position: &Position) -> Result<(), PortfolioError> {
        let sell_balance = *self.balances.get(&position.asset_sell).unwrap_or(&0f64);

        let is_sell_balance_enough = sell_balance.ge(&position.quantity_sell);
        if is_sell_balance_enough == false {
            return Err(PortfolioError::NotEnoughAssetSell(
                sell_balance,
                position.quantity_sell,
            ));
        }

        let buy_balance = *self.balances.get(&position.asset_buy).unwrap_or(&0f64);
        let buy_quantity = position.quantity_buy.unwrap_or(0f64);
        let is_buy_quantity_not_set = buy_quantity == 0f64;
        if is_buy_quantity_not_set == true {
            return Err(PortfolioError::BuyAssetQuantityNotSet());
        }

        self.balances.insert(
            position.asset_sell.to_owned(),
            sell_balance - position.quantity_sell,
        );
        self.balances
            .insert(position.asset_buy.to_owned(), buy_balance + buy_quantity);
        self.positions.push(position.to_owned());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn portfolio_new() {
        let mut balances = HashMap::new();
        let portfolio = Portfolio::new(balances, Vec::new());
        assert_eq!(portfolio.positions.len(), 0);
    }

    #[test]
    fn portfolio_add_position_success() {
        let sell_asset = Asset::new(Address::random(), String::from("LUSD"), 8);
        let buy_asset = Asset::new(Address::random(), String::from("ETH"), 8);
        let mut balances = HashMap::new();
        balances.insert(sell_asset.to_owned(), 1100f64);
        let mut portfolio = Portfolio::new(balances, Vec::new());

        let position = Position::new(
            Some(Utc::now()),
            sell_asset.to_owned(),
            buy_asset.to_owned(),
            500f64,
            Some(1f64),
            None,
        );
        let result = portfolio.add_position(&position);
        assert!(result.is_ok());
        assert_eq!(portfolio.positions.len(), 1);
        assert_eq!(portfolio.positions.get(0).unwrap(), &position);
        assert_eq!(portfolio.balances.get(&sell_asset).unwrap(), &600f64);
        assert_eq!(portfolio.balances.get(&buy_asset).unwrap(), &1f64);
    }

    #[test]
    fn portfolio_add_position_not_enough() {
        let sell_asset = Asset::new(Address::random(), String::from("LUSD"), 8);
        let buy_asset = Asset::new(Address::random(), String::from("ETH"), 8);
        let mut balances = HashMap::new();
        balances.insert(sell_asset.to_owned(), 499f64);
        let mut portfolio = Portfolio::new(balances, Vec::new());

        let position = Position::new(
            Some(Utc::now()),
            sell_asset.to_owned(),
            buy_asset.to_owned(),
            500f64,
            Some(1f64),
            None,
        );
        let result = portfolio.add_position(&position);
        assert_eq!(
            result.unwrap_err(),
            PortfolioError::NotEnoughAssetSell(499f64, 500f64)
        );
        assert_eq!(portfolio.positions.len(), 0);
        assert_eq!(portfolio.balances.get(&sell_asset).unwrap(), &499f64);
        assert_eq!(portfolio.balances.get(&buy_asset).unwrap_or(&0f64), &0f64);
    }

    #[test]
    fn portfolio_add_position_no_buy_quantity() {
        let sell_asset = Asset::new(Address::random(), String::from("LUSD"), 8);
        let buy_asset = Asset::new(Address::random(), String::from("ETH"), 8);
        let mut balances = HashMap::new();
        balances.insert(sell_asset.to_owned(), 1000f64);
        let mut portfolio = Portfolio::new(balances, Vec::new());

        let position = Position::new(
            Some(Utc::now()),
            sell_asset.to_owned(),
            buy_asset.to_owned(),
            500f64,
            None,
            None,
        );
        let result = portfolio.add_position(&position);
        assert_eq!(
            result.unwrap_err(),
            PortfolioError::BuyAssetQuantityNotSet()
        );
        assert_eq!(portfolio.positions.len(), 0);
        assert_eq!(portfolio.balances.get(&sell_asset).unwrap(), &1000f64);
        assert_eq!(portfolio.balances.get(&buy_asset).unwrap_or(&0f64), &0f64);
    }
}
*/
