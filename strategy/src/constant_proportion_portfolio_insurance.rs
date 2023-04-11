use crate::asset::Asset;
use crate::order::MarketOrder;
use thiserror::Error;

type Safe = f64;
type Risky = f64;

#[derive(Error, Debug, PartialEq)]
pub enum ConstantProportionPortfolioInsuranceError {
    #[error("An unexpected error happen...")]
    UnexpectedError,
}

/// Buy risky asset when the price increase and sell it for a safe asset when the price go down.
/// Define a min amount of capital to preserve and a multiplier to increase your risk exposure.
pub struct ConstantProportionPortfolioInsurance {
    risky_asset: Asset,
    safe_asset: Asset,
    multiplier: f64,
    min_safe_quantity: Safe,
}

impl ConstantProportionPortfolioInsurance {
    pub fn new(
        risky_asset: Asset,
        safe_asset: Asset,
        multiplier: f64,
        min_safe_quantity: Safe,
    ) -> Self {
        Self {
            risky_asset,
            safe_asset,
            multiplier,
            min_safe_quantity,
        }
    }

    pub fn check_new_order(
        &self,
        risky_hold_quantity: Risky,
        safe_hold_quantity: Safe,
        risky_price: Safe,
    ) -> Result<Option<MarketOrder>, ConstantProportionPortfolioInsuranceError> {
        let risky_hold_safe_value: Safe = (risky_hold_quantity * risky_price);
        let hold_quantity: Safe = risky_hold_safe_value + safe_hold_quantity;
        let cushion: Safe = hold_quantity - self.min_safe_quantity;
        if cushion <= 0f64 {
            if risky_hold_quantity == 0f64 {
                return Ok(None);
            }

            // Liquidate all risky asset
            return Ok(Some(MarketOrder::new(
                self.risky_asset.clone(),
                self.safe_asset.clone(),
                risky_hold_quantity,
            )));
        }

        let risky_new_hold_quantity: Safe = if cushion * self.multiplier > hold_quantity {
            hold_quantity
        } else {
            cushion * self.multiplier
        };
        let risky_delta = risky_new_hold_quantity - risky_hold_safe_value;
        match risky_delta {
            i if i == 0f64 => return Ok(None),
            i if i.is_sign_positive() => {
                // Increase risky asset exposure
                Ok(Some(MarketOrder::new(
                    self.safe_asset.clone(),
                    self.risky_asset.clone(),
                    risky_delta,
                )))
            }
            i if i.is_sign_negative() => {
                // Decrease risky asset exposure
                Ok(Some(MarketOrder::new(
                    self.risky_asset.clone(),
                    self.safe_asset.clone(),
                    risky_delta.abs() / risky_price,
                )))
            }
            _ => Err(ConstantProportionPortfolioInsuranceError::UnexpectedError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _constant_proportion_portfolio_insurance_new() -> ConstantProportionPortfolioInsurance {
        let risky_asset = Asset::new(String::from("ETH"), String::from("Ether"));
        let safe_asset = Asset::new(String::from("LUSD"), String::from("Liquity USD"));
        let multiplier = 3f64;
        let min_safe_quantity: Safe = 80f64;

        let cppi = ConstantProportionPortfolioInsurance::new(
            risky_asset,
            safe_asset,
            multiplier,
            min_safe_quantity,
        );
        cppi
    }

    #[test]
    fn constant_proportion_portfolio_insurance_new() {
        let cppi = _constant_proportion_portfolio_insurance_new();
        assert_eq!(cppi.min_safe_quantity, 80f64);
    }

    #[test]
    fn constant_proportion_portfolio_insurance_new_order_no_exposure() {
        let cppi = _constant_proportion_portfolio_insurance_new();
        let result = cppi.check_new_order(0f64, 100f64, 10f64);

        assert!(result.is_ok());
        let order = result.unwrap();
        assert!(order.is_some());
        let order = order.unwrap();

        assert_eq!(order.asset_sell, cppi.safe_asset);
        assert_eq!(order.asset_buy, cppi.risky_asset);
        assert_eq!(order.quantity_sell, 60f64);
    }

    #[test]
    fn constant_proportion_portfolio_insurance_new_order_not_enough_exposure() {
        let cppi = _constant_proportion_portfolio_insurance_new();
        let result = cppi.check_new_order(3f64, 70f64, 10f64);

        assert!(result.is_ok());
        let order = result.unwrap();
        assert!(order.is_some());
        let order = order.unwrap();

        assert_eq!(order.asset_sell, cppi.safe_asset);
        assert_eq!(order.asset_buy, cppi.risky_asset);
        assert_eq!(order.quantity_sell, 30f64);
    }

    #[test]
    fn constant_proportion_portfolio_insurance_new_order_not_enough_exposure_price() {
        let cppi = _constant_proportion_portfolio_insurance_new();
        let result = cppi.check_new_order(6f64, 40f64, 20f64);

        assert!(result.is_ok());
        let order = result.unwrap();
        assert!(order.is_some());
        let order = order.unwrap();

        assert_eq!(order.asset_sell, cppi.safe_asset);
        assert_eq!(order.asset_buy, cppi.risky_asset);
        assert_eq!(order.quantity_sell, 40f64);
    }

    #[test]
    fn constant_proportion_portfolio_insurance_new_order_reduce_exposure() {
        let cppi = _constant_proportion_portfolio_insurance_new();
        let result = cppi.check_new_order(6f64, 40f64, 9f64);

        assert!(result.is_ok());
        let order = result.unwrap();
        assert!(order.is_some());
        let order = order.unwrap();

        assert_eq!(order.asset_sell, cppi.risky_asset);
        assert_eq!(order.asset_buy, cppi.safe_asset);
        assert_eq!(order.quantity_sell, 12f64 / 9f64);
    }

    #[test]
    fn constant_proportion_portfolio_insurance_new_order_liquidation() {
        let cppi = _constant_proportion_portfolio_insurance_new();
        let result = cppi.check_new_order(6f64, 40f64, 1f64);

        assert!(result.is_ok());
        let order = result.unwrap();
        assert!(order.is_some());
        let order = order.unwrap();

        assert_eq!(order.asset_sell, cppi.risky_asset);
        assert_eq!(order.asset_buy, cppi.safe_asset);
        assert_eq!(order.quantity_sell, 6f64);
    }

    #[test]
    fn constant_proportion_portfolio_insurance_new_order_full_exposure() {
        let cppi = _constant_proportion_portfolio_insurance_new();
        let result = cppi.check_new_order(10f64, 0f64, 18f64);

        assert!(result.is_ok());
        let order = result.unwrap();
        assert!(order.is_none());
    }
}
