use crate::actor::*;
use crate::market::{Tick, TickError};
use crate::mul_div::*;
use ethers::types::U64;
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum RunnerError {
    #[error("Price increment should be greater than 0 ({0})")]
    PriceIncrementCantBeZeroNegative(U64),
    #[error("Duration between trade range ms should be greater than zero and first entry smaller than second ({0} => {1})")]
    DurationBetweenTradeRangeMsIncorrect(u64, u64),
    #[error("Duration between market state range ms should be greater than zero and first entry smaller than second ({0} => {1})")]
    DurationBetweenMarketStateRangeMsIncorrect(u64, u64),
    #[error(
    "Volume base range should be greater than zero and first entry smaller than second ({0} => {1})"
    )]
    VolumeBaseRangeIncorrect(U64, U64),
    #[error(
    "Liquidity change by tick range should be greater than zero and first entry smaller than second ({0} => {1})"
    )]
    LiquidityChangeByTickRangeIncorrect(U64, U64),
    #[error("Actor Liquidity amplifier should be greater than zero ({0})")]
    ActorLiquidityAmplifierCantBeZeroNegative(U64),
    #[error("Current price should be greater than price increment ({0} > {1})")]
    CurrentPriceCantBeLessThanIncrement(U64, U64),
    #[error("Market volume muldiv by actor amplifier overflow ({0} muldiv {1})")]
    MarketVolumeMulDivAmplifierOverflow(U64, U64),
    #[error("Limit volume muldiv by actor amplifier overflow ({0} muldiv {1})")]
    LimitVolumeMulDivAmplifierOverflow(U64, U64),
    #[error("Tick error {0}")]
    Tick(TickError),
    #[error("Actors error {0}")]
    Actor(ActorsError),
}

/// Config to run a market simulation
pub struct Runner {
    pub price_increment: U64,
    pub duration_between_trade_range_ms: (u64, u64),
    pub duration_between_market_state_range_ms: (u64, u64),
    pub volume_base_range: (U64, U64),
    pub liquidity_change_by_tick_range: (U64, U64),
    pub actor_liquidity_amplifier_x1_000_000: U64,
}

impl Runner {
    pub fn new(
        price_increment: U64,
        duration_between_trade_range_ms: (u64, u64),
        duration_between_market_state_range_ms: (u64, u64),
        volume_base_range: (U64, U64),
        liquidity_change_by_tick_range: (U64, U64),
        actor_liquidity_amplifier_x1_000_000: U64,
    ) -> Result<Self, RunnerError> {
        if price_increment.is_zero() {
            return Err(RunnerError::PriceIncrementCantBeZeroNegative(
                price_increment,
            ));
        }

        let is_duration_gt_zero = duration_between_trade_range_ms.0 > 0;
        let is_range_ascending =
            duration_between_trade_range_ms.0 < duration_between_trade_range_ms.1;
        if !(is_duration_gt_zero && is_range_ascending) {
            return Err(RunnerError::DurationBetweenTradeRangeMsIncorrect(
                duration_between_trade_range_ms.0,
                duration_between_trade_range_ms.1,
            ));
        }

        let is_duration_gt_zero = duration_between_market_state_range_ms.0 > 0;
        let is_range_ascending =
            duration_between_market_state_range_ms.0 < duration_between_market_state_range_ms.1;
        if !(is_duration_gt_zero && is_range_ascending) {
            return Err(RunnerError::DurationBetweenMarketStateRangeMsIncorrect(
                duration_between_market_state_range_ms.0,
                duration_between_market_state_range_ms.1,
            ));
        }

        let is_range_ascending = volume_base_range.0 < volume_base_range.1;
        if !(!volume_base_range.0.is_zero() && is_range_ascending) {
            return Err(RunnerError::VolumeBaseRangeIncorrect(
                volume_base_range.0,
                volume_base_range.1,
            ));
        }

        let is_range_ascending =
            liquidity_change_by_tick_range.0 < liquidity_change_by_tick_range.1;
        if !(!liquidity_change_by_tick_range.0.is_zero() && is_range_ascending) {
            return Err(RunnerError::LiquidityChangeByTickRangeIncorrect(
                liquidity_change_by_tick_range.0,
                liquidity_change_by_tick_range.1,
            ));
        }

        if actor_liquidity_amplifier_x1_000_000.is_zero() {
            return Err(RunnerError::ActorLiquidityAmplifierCantBeZeroNegative(
                actor_liquidity_amplifier_x1_000_000,
            ));
        }

        Ok(Self {
            price_increment,
            duration_between_trade_range_ms,
            duration_between_market_state_range_ms,
            volume_base_range,
            liquidity_change_by_tick_range,
            actor_liquidity_amplifier_x1_000_000,
        })
    }
    pub fn default() -> Self {
        Runner::new(
            U64::from(1) * U64::exp10(5),
            (15, 30_000),
            (14 * 24 * 60 * 60 * 1000, 90 * 24 * 60 * 60 * 1000),
            (U64::from(1) * U64::exp10(6), U64::from(100) * U64::exp10(6)),
            (U64::from(1) * U64::exp10(6), U64::from(100) * U64::exp10(6)),
            U64::from(1_005_000),
        )
        .unwrap()
    }
    pub fn run(
        &mut self,
        mut current_time_ms: u64,
        end_time_ms: u64,
        mut current_price: U64,
    ) -> Result<Vec<Tick>, RunnerError> {
        let mut rng = thread_rng();
        let mut ticks: Vec<Tick> = Vec::new();

        while current_time_ms < end_time_ms {
            let current_actor_power =
                ActorPower::new(ActorPowerState::EQUAL, ActorPowerState::EQUAL);
            let current_duration_market_state_ms = rng
                .gen_range(
                    self.duration_between_market_state_range_ms.0
                        ..=self.duration_between_market_state_range_ms.1,
                )
                .min(end_time_ms - current_time_ms);
            ticks.append(&mut Runner::make_ticks_for_actor_power(
                self,
                &mut rng,
                current_time_ms,
                current_price,
                current_duration_market_state_ms,
                &current_actor_power,
            )?);

            current_time_ms += current_duration_market_state_ms;
            if let Some(last) = ticks.last() {
                current_price = last.price;
            }
        }

        Ok(ticks)
    }

    fn make_ticks_for_actor_power(
        _runner: &Runner,
        _rng: &mut ThreadRng,
        _current_time_ms: u64,
        _current_price: U64,
        _current_duration_market_state_ms: u64,
        _current_actor_power: &ActorPower,
    ) -> Result<Vec<Tick>, RunnerError> {
        let mut ticks: Vec<Tick> = Vec::new();
        let end_time_market_state_ms = _current_time_ms + _current_duration_market_state_ms;
        let mut current_time_market_state_ms = _current_time_ms;
        let mut current_price = _current_price;

        while current_time_market_state_ms < end_time_market_state_ms {
            let is_buy = _rng.gen_bool(0.5);
            let actors = Runner::make_actors(_runner, _rng, _current_actor_power, is_buy)?;

            ticks.append(&mut Runner::make_ticks_for_actors(
                _runner,
                &actors,
                current_time_market_state_ms,
                current_price,
                is_buy,
            )?);

            if let Some(tick) = ticks.last() {
                current_price = tick.price;
            };
            current_time_market_state_ms += _rng.gen_range(
                _runner.duration_between_trade_range_ms.0
                    ..=_runner.duration_between_trade_range_ms.1,
            );
        }

        Ok(ticks)
    }

    pub fn make_ticks_for_actors(
        _runner: &Runner,
        _actors: &Actors,
        _current_time_ms: u64,
        _current_price: U64,
        is_buy: bool,
    ) -> Result<Vec<Tick>, RunnerError> {
        let is_current_price_gt_price_increment = _current_price > _runner.price_increment;
        if !is_current_price_gt_price_increment {
            return Err(RunnerError::CurrentPriceCantBeLessThanIncrement(
                _current_price,
                _runner.price_increment,
            ));
        }

        let mut ticks: Vec<Tick> = Vec::new();
        let mut market_volume_left = _actors.market_volume;
        let mut limit_volume_left = _actors.limit_volume_by_tick;
        let mut current_price = _current_price;

        while market_volume_left > U64::zero() && current_price > _runner.price_increment {
            let is_liquidity_consumed = market_volume_left > limit_volume_left;
            let volume = market_volume_left.min(limit_volume_left);
            market_volume_left -= volume;

            ticks.push(
                Tick::new(current_price, _current_time_ms, volume, is_buy)
                    .map_err(RunnerError::Tick)?,
            );
            if is_liquidity_consumed {
                current_price = if is_buy {
                    current_price + _runner.price_increment
                } else {
                    current_price - _runner.price_increment
                };
                limit_volume_left += _actors.limit_volume_change_by_tick;
            }
        }
        Ok(ticks)
    }

    pub fn make_actors(
        _runner: &Runner,
        _rng: &mut ThreadRng,
        _current_actor_power: &ActorPower,
        _is_buy: bool,
    ) -> Result<Actors, RunnerError> {
        let actor_power = if _is_buy {
            &_current_actor_power.market_buyer_vs_limit_seller
        } else {
            &_current_actor_power.market_seller_vs_limit_buyer
        };
        let mut actors = Actors::new(
            U64::from(_rng.gen_range(
                _runner.volume_base_range.0.as_u64()..=_runner.volume_base_range.1.as_u64(),
            )),
            U64::from(_rng.gen_range(
                _runner.volume_base_range.0.as_u64()..=_runner.volume_base_range.1.as_u64(),
            )),
            U64::from(_rng.gen_range(
                _runner.liquidity_change_by_tick_range.0.as_u64()
                    ..=_runner.liquidity_change_by_tick_range.1.as_u64(),
            )),
        )
        .map_err(RunnerError::Actor)?;

        match actor_power {
            ActorPowerState::LESS => {
                actors.limit_volume_by_tick = mul_div_u64(
                    actors.limit_volume_by_tick,
                    _runner.actor_liquidity_amplifier_x1_000_000,
                    U64::exp10(6),
                )
                .ok_or(RunnerError::LimitVolumeMulDivAmplifierOverflow(
                    actors.limit_volume_by_tick,
                    _runner.actor_liquidity_amplifier_x1_000_000,
                ))?;
                actors.limit_volume_change_by_tick = mul_div_u64(
                    actors.limit_volume_change_by_tick,
                    _runner.actor_liquidity_amplifier_x1_000_000,
                    U64::exp10(6),
                )
                .ok_or(RunnerError::LimitVolumeMulDivAmplifierOverflow(
                    actors.limit_volume_change_by_tick,
                    _runner.actor_liquidity_amplifier_x1_000_000,
                ))?;
            }
            ActorPowerState::EQUAL => {}
            ActorPowerState::GREATER => {
                actors.market_volume = mul_div_u64(
                    actors.market_volume,
                    _runner.actor_liquidity_amplifier_x1_000_000,
                    U64::exp10(6),
                )
                .ok_or(RunnerError::MarketVolumeMulDivAmplifierOverflow(
                    actors.market_volume,
                    _runner.actor_liquidity_amplifier_x1_000_000,
                ))?;
            }
        }
        Ok(actors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runner_new() {
        let runner = Runner::default();
        assert_eq!(runner.price_increment, U64::from(1) * U64::exp10(5));
        assert_eq!(runner.duration_between_trade_range_ms, (15, 30_000));
        assert_eq!(
            runner.duration_between_market_state_range_ms,
            (14 * 24 * 60 * 60 * 1000, 90 * 24 * 60 * 60 * 1000)
        );
        assert_eq!(
            runner.volume_base_range,
            (U64::from(1) * U64::exp10(6), U64::from(100) * U64::exp10(6))
        );
        assert_eq!(
            runner.liquidity_change_by_tick_range,
            (U64::from(1) * U64::exp10(6), U64::from(100) * U64::exp10(6))
        );
        assert_eq!(
            runner.actor_liquidity_amplifier_x1_000_000,
            U64::from(1_005_000)
        );
    }

    fn _assert_equal_in_range(actors: Result<Actors, RunnerError>, runner: &Runner) {
        assert!(actors.is_ok());
        let actors = actors.unwrap();
        assert!(
            actors.market_volume >= runner.volume_base_range.0
                && actors.market_volume <= runner.volume_base_range.1
        );
        assert!(
            actors.limit_volume_by_tick >= runner.volume_base_range.0
                && actors.limit_volume_by_tick <= runner.volume_base_range.1
        );
        assert!(
            actors.limit_volume_change_by_tick >= runner.liquidity_change_by_tick_range.0
                && actors.limit_volume_change_by_tick <= runner.liquidity_change_by_tick_range.1
        );
    }

    fn _assert_greater_in_range(actors: Result<Actors, RunnerError>, runner: &Runner) {
        assert!(actors.is_ok());
        let actors = actors.unwrap();

        let max_0 = mul_div_u64(
            runner.volume_base_range.0,
            runner.actor_liquidity_amplifier_x1_000_000,
            U64::exp10(6),
        );
        assert!(max_0.is_some());
        let max_0 = max_0.unwrap();
        let max_1 = mul_div_u64(
            runner.volume_base_range.1,
            runner.actor_liquidity_amplifier_x1_000_000,
            U64::exp10(6),
        );
        assert!(max_1.is_some());
        let max_1 = max_1.unwrap();
        assert!(actors.market_volume >= max_0 && actors.market_volume <= max_1);

        assert!(
            actors.limit_volume_by_tick >= runner.volume_base_range.0
                && actors.limit_volume_by_tick <= runner.volume_base_range.1
        );
        assert!(
            actors.limit_volume_by_tick >= runner.liquidity_change_by_tick_range.0
                && actors.limit_volume_by_tick <= runner.liquidity_change_by_tick_range.1
        );
    }

    fn _assert_less_in_range(actors: Result<Actors, RunnerError>, runner: &Runner) {
        assert!(actors.is_ok());
        let actors = actors.unwrap();

        assert!(
            actors.market_volume >= runner.volume_base_range.0
                && actors.market_volume <= runner.volume_base_range.1
        );

        let max_0 = mul_div_u64(
            runner.volume_base_range.0,
            runner.actor_liquidity_amplifier_x1_000_000,
            U64::exp10(6),
        );
        assert!(max_0.is_some());
        let max_0 = max_0.unwrap();
        let max_1 = mul_div_u64(
            runner.volume_base_range.1,
            runner.actor_liquidity_amplifier_x1_000_000,
            U64::exp10(6),
        );
        assert!(max_1.is_some());
        let max_1 = max_1.unwrap();
        assert!(actors.limit_volume_by_tick >= max_0 && actors.limit_volume_by_tick <= max_1);

        let max_0 = mul_div_u64(
            runner.liquidity_change_by_tick_range.0,
            runner.actor_liquidity_amplifier_x1_000_000,
            U64::exp10(6),
        );
        assert!(max_0.is_some());
        let max_0 = max_0.unwrap();
        let max_1 = mul_div_u64(
            runner.liquidity_change_by_tick_range.1,
            runner.actor_liquidity_amplifier_x1_000_000,
            U64::exp10(6),
        );
        assert!(max_1.is_some());
        let max_1 = max_1.unwrap();
        assert!(actors.limit_volume_by_tick >= max_0 && actors.limit_volume_by_tick <= max_1);
    }

    #[test]
    fn maker_actors_success() {
        let mut rng = thread_rng();
        let runner = Runner::default();
        for _i in 0..1_000 {
            let is_buy = rng.gen_bool(0.5);
            let actors = Runner::make_actors(
                &runner,
                &mut rng,
                &ActorPower::new(ActorPowerState::EQUAL, ActorPowerState::EQUAL),
                is_buy,
            );
            _assert_equal_in_range(actors, &runner);
            let actors = Runner::make_actors(
                &runner,
                &mut rng,
                &ActorPower::new(ActorPowerState::GREATER, ActorPowerState::GREATER),
                is_buy,
            );
            _assert_greater_in_range(actors, &runner);
            let actors = Runner::make_actors(
                &runner,
                &mut rng,
                &ActorPower::new(ActorPowerState::LESS, ActorPowerState::LESS),
                is_buy,
            );
            _assert_less_in_range(actors, &runner);

            let actors = Runner::make_actors(
                &runner,
                &mut rng,
                &ActorPower::new(ActorPowerState::EQUAL, ActorPowerState::LESS),
                is_buy,
            );
            if is_buy {
                _assert_equal_in_range(actors, &runner);
            } else {
                _assert_less_in_range(actors, &runner);
            }
            let actors = Runner::make_actors(
                &runner,
                &mut rng,
                &ActorPower::new(ActorPowerState::EQUAL, ActorPowerState::GREATER),
                is_buy,
            );
            if is_buy {
                _assert_equal_in_range(actors, &runner);
            } else {
                _assert_greater_in_range(actors, &runner);
            }
            let actors = Runner::make_actors(
                &runner,
                &mut rng,
                &ActorPower::new(ActorPowerState::GREATER, ActorPowerState::EQUAL),
                is_buy,
            );
            if is_buy {
                _assert_greater_in_range(actors, &runner);
            } else {
                _assert_equal_in_range(actors, &runner);
            }
            let actors = Runner::make_actors(
                &runner,
                &mut rng,
                &ActorPower::new(ActorPowerState::GREATER, ActorPowerState::LESS),
                is_buy,
            );
            if is_buy {
                _assert_greater_in_range(actors, &runner);
            } else {
                _assert_less_in_range(actors, &runner);
            }
            let actors = Runner::make_actors(
                &runner,
                &mut rng,
                &ActorPower::new(ActorPowerState::LESS, ActorPowerState::EQUAL),
                is_buy,
            );
            if is_buy {
                _assert_less_in_range(actors, &runner);
            } else {
                _assert_equal_in_range(actors, &runner);
            }
            let actors = Runner::make_actors(
                &runner,
                &mut rng,
                &ActorPower::new(ActorPowerState::LESS, ActorPowerState::GREATER),
                is_buy,
            );
            if is_buy {
                _assert_less_in_range(actors, &runner);
            } else {
                _assert_greater_in_range(actors, &runner);
            }
        }
    }

    #[test]
    fn make_ticks_for_actors_success() {
        let runner = Runner::default();
        let actors = Actors::default();
        let current_time_ms: u64 = 42;
        let current_price: U64 = U64::from(1_000) * U64::exp10(6);
        let is_buy = true;
        let ticks =
            Runner::make_ticks_for_actors(&runner, &actors, current_time_ms, current_price, is_buy);
        assert!(ticks.is_ok());
        let ticks = ticks.unwrap();
        assert_eq!(ticks.len(), 1);
        let tick = ticks.first().unwrap();
        assert_eq!(tick.price, current_price);
        assert_eq!(tick.time, current_time_ms);
        assert_eq!(tick.volume, actors.market_volume);
        assert_eq!(tick.is_up, is_buy);

        let is_buy = false;
        let ticks =
            Runner::make_ticks_for_actors(&runner, &actors, current_time_ms, current_price, is_buy);
        assert!(ticks.is_ok());
        let ticks = ticks.unwrap();
        assert_eq!(ticks.len(), 1);
        let tick = ticks.first().unwrap();
        assert_eq!(tick.price, current_price);
        assert_eq!(tick.time, current_time_ms);
        assert_eq!(tick.volume, actors.market_volume);
        assert_eq!(tick.is_up, is_buy);
    }

    #[test]
    fn make_ticks_for_actors_multiple_tick() {
        let runner = Runner::default();
        let actors = Actors::new(
            U64::from(220) * U64::exp10(6),
            U64::from(100) * U64::exp10(6),
            U64::from(10) * U64::exp10(6),
        )
        .unwrap();
        let current_time_ms: u64 = 42;
        let current_price = U64::from(1_000) * U64::exp10(6);
        let is_buy = true;
        let ticks =
            Runner::make_ticks_for_actors(&runner, &actors, current_time_ms, current_price, is_buy);
        assert!(ticks.is_ok());
        let ticks = ticks.unwrap();
        assert_eq!(ticks.len(), 3);
        let tick = ticks.get(0).unwrap();
        assert_eq!(tick.price, current_price);
        assert_eq!(tick.time, current_time_ms);
        assert_eq!(tick.volume, U64::from(100) * U64::exp10(6));
        assert_eq!(tick.is_up, is_buy);
        let tick = ticks.get(1).unwrap();
        assert_eq!(tick.price, current_price + runner.price_increment);
        assert_eq!(tick.time, current_time_ms);
        assert_eq!(tick.volume, U64::from(110) * U64::exp10(6));
        assert_eq!(tick.is_up, is_buy);
        let tick = ticks.get(2).unwrap();
        assert_eq!(tick.price, current_price + (runner.price_increment * 2));
        assert_eq!(tick.time, current_time_ms);
        assert_eq!(tick.volume, U64::from(10) * U64::exp10(6));
        assert_eq!(tick.is_up, is_buy);
    }
}
