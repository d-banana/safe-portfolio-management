use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ActorsError {
    #[error("Market volume should be greater than zero ({0})")]
    MarketVolumeCantBeZeroNegative(f64),
    #[error("Limit volume by tick should be greater than zero ({0})")]
    LimitVolumeByTickCantBeZeroNegative(f64),
    #[error("Limit volume change by tick should be greater than zero ({0})")]
    LimitVolumeChangeByTickCantBeZeroNegative(f64),
}

/// Define a taker buyer/seller vs maker seller/buyer
/// market_volume is how much will be buy/sell
/// limit_volume_by_tick is how much the take is willing to sell/buy at best price
/// if there is not enough volume to absorb the taker volume we can move to the next tick,
/// and increase the limit_volume_by_tick with limit_volume_change_by_tick
pub struct Actors {
    pub market_volume: f64,
    pub limit_volume_by_tick: f64,
    pub limit_volume_change_by_tick: f64,
}

impl Actors {
    pub fn new(
        market_volume: f64,
        limit_volume_by_tick: f64,
        limit_volume_change_by_tick: f64,
    ) -> Result<Self, ActorsError> {
        let is_market_volume_gt_zero = market_volume > 0.0;
        if !is_market_volume_gt_zero {
            return Err(ActorsError::MarketVolumeCantBeZeroNegative(market_volume));
        }
        let is_limit_volume_by_tick_gt_zero = limit_volume_by_tick > 0.0;
        if !is_limit_volume_by_tick_gt_zero {
            return Err(ActorsError::LimitVolumeByTickCantBeZeroNegative(
                limit_volume_by_tick,
            ));
        }
        let is_limit_volume_change_by_tick_gt_zero = limit_volume_change_by_tick > 0.0;
        if !is_limit_volume_change_by_tick_gt_zero {
            return Err(ActorsError::LimitVolumeChangeByTickCantBeZeroNegative(
                limit_volume_change_by_tick,
            ));
        }

        Ok(Self {
            market_volume,
            limit_volume_by_tick,
            limit_volume_change_by_tick,
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ActorPowerState {
    LESS,
    EQUAL,
    GREATER,
}

pub struct ActorPower {
    pub market_buyer_vs_limit_seller: ActorPowerState,
    pub market_seller_vs_limit_buyer: ActorPowerState,
}

impl ActorPower {
    pub fn new(
        market_buyer_vs_limit_seller: ActorPowerState,
        market_seller_vs_limit_buyer: ActorPowerState,
    ) -> Self {
        Self {
            market_buyer_vs_limit_seller,
            market_seller_vs_limit_buyer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actors_new() {
        let actors = Actors::new(1.0, 1.1, 0.1);
        assert!(actors.is_ok());
        let actors = actors.unwrap();
        assert_eq!(actors.market_volume, 1.0);
        assert_eq!(actors.limit_volume_by_tick, 1.1);
        assert_eq!(actors.limit_volume_change_by_tick, 0.1);
    }

    #[test]
    fn actor_power_new() {
        let actor_power = ActorPower::new(ActorPowerState::EQUAL, ActorPowerState::GREATER);
        assert_eq!(
            actor_power.market_buyer_vs_limit_seller,
            ActorPowerState::EQUAL
        );
        assert_eq!(
            actor_power.market_seller_vs_limit_buyer,
            ActorPowerState::GREATER
        );
    }
}
