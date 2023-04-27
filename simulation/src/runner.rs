use crate::actor::*;
use crate::market::{Tick, TickError};
use crate::mul_div::*;
use ethers::types::{I256, U64};
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
    #[error("Duration moving average tick should be greater than 0 ({0})")]
    DurationMovingAverageTickCantBeZeroNegative(usize),
    #[error("Moving average muldiv by old lend overflow ({0} muldiv {1})")]
    MovingAverageMulDivLenOverflow(I256, U64),
    #[error("Last tick doesn't have moving average")]
    LastTickMovingAverageNone(),
    #[error("Moving average can't be negative({0})")]
    MovingAverageIsNegative(I256),
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
    pub duration_moving_average_tick: usize,
}

impl Runner {
    pub fn new(
        price_increment: U64,
        duration_between_trade_range_ms: (u64, u64),
        duration_between_market_state_range_ms: (u64, u64),
        volume_base_range: (U64, U64),
        liquidity_change_by_tick_range: (U64, U64),
        actor_liquidity_amplifier_x1_000_000: U64,
        duration_moving_average_tick: usize,
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

        let is_duration_ma_tick_gt_zero = duration_moving_average_tick > 0;
        if !is_duration_ma_tick_gt_zero {
            return Err(RunnerError::DurationMovingAverageTickCantBeZeroNegative(
                duration_moving_average_tick,
            ));
        }

        Ok(Self {
            price_increment,
            duration_between_trade_range_ms,
            duration_between_market_state_range_ms,
            volume_base_range,
            liquidity_change_by_tick_range,
            actor_liquidity_amplifier_x1_000_000,
            duration_moving_average_tick,
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
            1_000_000,
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
                &ticks,
            )?);

            current_time_ms += current_duration_market_state_ms;
            if let Some(last) = ticks.last() {
                current_price = last.price;
            }
        }

        Ok(ticks)
    }

    pub fn make_ticks_for_actor_power(
        _runner: &Runner,
        _rng: &mut ThreadRng,
        _current_time_ms: u64,
        _current_price: U64,
        _current_duration_market_state_ms: u64,
        _current_actor_power: &ActorPower,
        _ticks: &Vec<Tick>,
    ) -> Result<Vec<Tick>, RunnerError> {
        let mut ticks: Vec<Tick> = Vec::new();
        let end_time_market_state_ms = _current_time_ms + _current_duration_market_state_ms;
        let mut current_time_market_state_ms = _current_time_ms;
        let mut current_price = _current_price;

        while current_time_market_state_ms < end_time_market_state_ms {
            let is_buy = _rng.gen_bool(0.5);
            let actors = Runner::make_actors(_runner, _rng, _current_actor_power, is_buy)?;
            for tick in Runner::make_ticks_for_actors(
                _runner,
                &actors,
                current_time_market_state_ms,
                current_price,
                is_buy,
            )? {
                ticks.push(Runner::make_sliding_moving_average_from_ticks(
                    _runner, _ticks, &ticks, &tick,
                )?);
            }

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
                Tick::new(current_price, _current_time_ms, volume, is_buy, None, None)
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

    pub fn make_sliding_moving_average_from_ticks(
        _runner: &Runner,
        _old_ticks: &Vec<Tick>,
        _new_ticks: &Vec<Tick>,
        _new_tick: &Tick,
    ) -> Result<Tick, RunnerError> {
        let tick_len = _runner
            .duration_moving_average_tick
            .min(_old_ticks.len() + _new_ticks.len());

        let is_ticks_zero = tick_len == 0;
        if is_ticks_zero {
            return Runner::make_sliding_moving_average(_runner, &None, &None, tick_len, _new_tick);
        }

        let mut first_tick = None;
        if _new_ticks.len() >= _runner.duration_moving_average_tick {
            first_tick = _new_ticks.get(_new_ticks.len() - _runner.duration_moving_average_tick);
        } else if _new_ticks.len() + _old_ticks.len() >= _runner.duration_moving_average_tick {
            first_tick = _old_ticks
                .get(_old_ticks.len() - (_runner.duration_moving_average_tick - _new_ticks.len()));
        } else if _old_ticks.is_empty() {
            first_tick = _new_ticks.first();
        } else {
            first_tick = _old_ticks.first();
        }

        let mut last_tick = None;
        if !_new_ticks.is_empty() {
            last_tick = _new_ticks.last();
        } else {
            last_tick = _old_ticks.last();
        }

        Runner::make_sliding_moving_average(_runner, &first_tick, &last_tick, tick_len, _new_tick)
    }

    pub fn make_sliding_moving_average(
        _runner: &Runner,
        _first_tick: &Option<&Tick>,
        _last_tick: &Option<&Tick>,
        _tick_len: usize,
        _new_tick: &Tick,
    ) -> Result<Tick, RunnerError> {
        let is_ticks_zero = _tick_len == 0;
        if is_ticks_zero {
            let mut tick = _new_tick.clone();
            tick.moving_average = Some(tick.price);
            return Ok(tick);
        }
        let mut moving_average: I256 = I256::from(_new_tick.price.as_u64());

        if _tick_len == _runner.duration_moving_average_tick {
            if let Some(tick) = _first_tick {
                moving_average -= I256::from(tick.price.as_u64());
            }
        } else if let Some(tick) = _last_tick {
            moving_average -= I256::from(
                tick.moving_average
                    .ok_or(RunnerError::LastTickMovingAverageNone())?
                    .as_u64(),
            );
        }

        let old_ma_len =
            U64::from((_tick_len + 1).min(_runner.duration_moving_average_tick)) * U64::exp10(6);
        moving_average = mul_div_i256(
            moving_average,
            I256::exp10(6),
            I256::from(old_ma_len.as_u64()),
        )
        .ok_or(RunnerError::MovingAverageMulDivLenOverflow(
            moving_average,
            old_ma_len,
        ))?;

        moving_average += I256::from(
            _last_tick
                .clone()
                .unwrap()
                .moving_average
                .ok_or(RunnerError::LastTickMovingAverageNone())?
                .as_u64(),
        );
        let mut tick = _new_tick.clone();
        if moving_average.is_negative() {
            return Err(RunnerError::MovingAverageIsNegative(moving_average));
        }
        tick.moving_average = Some(U64::from(moving_average.into_sign_and_abs().1.as_u64()));
        Ok(tick)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::ops::Add;

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
        assert_eq!(runner.duration_moving_average_tick, 10_000);
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
    fn make_actors_success() {
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
        // SELL
        let is_buy = false;
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
        assert_eq!(tick.price, current_price - runner.price_increment);
        assert_eq!(tick.time, current_time_ms);
        assert_eq!(tick.volume, U64::from(110) * U64::exp10(6));
        assert_eq!(tick.is_up, is_buy);
        let tick = ticks.get(2).unwrap();
        assert_eq!(tick.price, current_price - (runner.price_increment * 2));
        assert_eq!(tick.time, current_time_ms);
        assert_eq!(tick.volume, U64::from(10) * U64::exp10(6));
        assert_eq!(tick.is_up, is_buy);
    }

    #[test]
    fn make_ticks_for_actor_power_success() {
        let mut rng = thread_rng();
        let runner = Runner::default();
        let actors = Actors::new(
            U64::from(220) * U64::exp10(6),
            U64::from(100) * U64::exp10(6),
            U64::from(10) * U64::exp10(6),
        )
        .unwrap();
        let current_time_ms: u64 = 42;
        let current_price = U64::from(1_000) * U64::exp10(6);
        let current_duration_market_state_ms = 1_000_000;
        let current_actor_power = ActorPower::new(ActorPowerState::EQUAL, ActorPowerState::EQUAL);
        let ticks = Runner::make_ticks_for_actor_power(
            &runner,
            &mut rng,
            current_time_ms,
            current_price,
            current_duration_market_state_ms,
            &current_actor_power,
            &vec![],
        );

        assert!(ticks.is_ok());
        let ticks = ticks.unwrap();
        let first_tick = ticks.first().unwrap();
        assert_eq!(first_tick.time, current_time_ms);
        let last_tick = ticks.last().unwrap();
        assert!(last_tick.time <= current_time_ms + current_duration_market_state_ms)
    }

    fn check_average_price(
        actor_power_a: &ActorPower,
        actor_power_b: &ActorPower,
        avg_map: &HashMap<ActorPower, U64>,
        is_a_gt_b: bool,
    ) {
        let a = avg_map.get(&actor_power_a);
        let b = avg_map.get(&actor_power_b);
        assert!(a.is_some());
        assert!(b.is_some());
        let a = a.unwrap();
        let b = b.unwrap();
        if is_a_gt_b {
            assert!(a > b);
        } else {
            assert!(a < b);
        }
    }

    #[test]
    fn make_ticks_for_actor_power_trend() {
        let mut rng = thread_rng();
        let runner = Runner::new(
            U64::from(1) * U64::exp10(5),
            (15, 30_000),
            (14 * 24 * 60 * 60 * 1000, 90 * 24 * 60 * 60 * 1000),
            (U64::from(1) * U64::exp10(6), U64::from(100) * U64::exp10(6)),
            (U64::from(1) * U64::exp10(6), U64::from(100) * U64::exp10(6)),
            U64::from(1_008_000),
            10_000,
        )
        .unwrap();
        let actors = Actors::new(
            U64::from(220) * U64::exp10(6),
            U64::from(100) * U64::exp10(6),
            U64::from(10) * U64::exp10(6),
        )
        .unwrap();
        let current_time_ms: u64 = 42;
        let current_price = U64::from(1_000) * U64::exp10(6);
        let current_duration_market_state_ms = 14 * 24 * 60 * 60 * 1000;
        let mut prices_map: HashMap<ActorPower, Vec<U64>> = HashMap::new();
        let actor_powers = vec![
            ActorPower::new(ActorPowerState::EQUAL, ActorPowerState::EQUAL), // 0
            ActorPower::new(ActorPowerState::GREATER, ActorPowerState::GREATER), // 1
            ActorPower::new(ActorPowerState::LESS, ActorPowerState::LESS),   // 2
            ActorPower::new(ActorPowerState::EQUAL, ActorPowerState::GREATER), // 3
            ActorPower::new(ActorPowerState::EQUAL, ActorPowerState::LESS),  // 4
            ActorPower::new(ActorPowerState::GREATER, ActorPowerState::EQUAL), // 5
            ActorPower::new(ActorPowerState::GREATER, ActorPowerState::LESS), // 6
            ActorPower::new(ActorPowerState::LESS, ActorPowerState::EQUAL),  // 7
            ActorPower::new(ActorPowerState::LESS, ActorPowerState::GREATER), // 8
        ];

        for actor_power in &actor_powers {
            for _ in 0..4 {
                let current_actor_power = actor_power;

                let ticks = Runner::make_ticks_for_actor_power(
                    &runner,
                    &mut rng,
                    current_time_ms,
                    current_price,
                    current_duration_market_state_ms,
                    current_actor_power,
                    &vec![],
                );
                assert!(ticks.is_ok());
                let ticks = ticks.unwrap();
                let last_tick = ticks.last().unwrap();
                let prices = prices_map.get(actor_power);
                if let None = prices {
                    prices_map.insert(actor_power.to_owned(), vec![last_tick.price]);
                    continue;
                };
                if let Some(prices) = prices {
                    let mut prices = prices.to_owned();
                    prices.push(last_tick.price);
                    prices_map.insert(actor_power.to_owned(), prices);
                    continue;
                };
            }
        }
        let mut avg_map: HashMap<ActorPower, U64> = HashMap::new();
        for (actor_power, prices) in prices_map.iter() {
            let mut sum: U64 = U64::zero();
            for price in prices {
                sum += price.to_owned();
            }
            avg_map.insert(actor_power.to_owned(), sum / prices.len());
        }

        let crabs = vec![
            ActorPower::new(ActorPowerState::EQUAL, ActorPowerState::EQUAL),
            ActorPower::new(ActorPowerState::GREATER, ActorPowerState::GREATER),
            ActorPower::new(ActorPowerState::LESS, ActorPowerState::LESS),
        ];
        let ups = vec![
            ActorPower::new(ActorPowerState::EQUAL, ActorPowerState::LESS),
            ActorPower::new(ActorPowerState::GREATER, ActorPowerState::EQUAL),
            ActorPower::new(ActorPowerState::GREATER, ActorPowerState::LESS),
        ];
        let downs = vec![
            ActorPower::new(ActorPowerState::LESS, ActorPowerState::EQUAL),
            ActorPower::new(ActorPowerState::EQUAL, ActorPowerState::GREATER),
            ActorPower::new(ActorPowerState::LESS, ActorPowerState::GREATER),
        ];
        for crab in &crabs {
            for up in &ups {
                check_average_price(crab, up, &avg_map, false);
            }
            for down in &downs {
                check_average_price(crab, down, &avg_map, true);
            }
        }
        check_average_price(ups.get(0).unwrap(), ups.get(1).unwrap(), &avg_map, false);
        check_average_price(ups.get(1).unwrap(), ups.get(2).unwrap(), &avg_map, false);
        check_average_price(downs.get(0).unwrap(), downs.get(1).unwrap(), &avg_map, true);
        check_average_price(downs.get(1).unwrap(), downs.get(2).unwrap(), &avg_map, true);
    }

    #[test]
    fn make_moving_average_success() {
        let runner = Runner::default();
        let tick = Runner::make_sliding_moving_average(
            &runner,
            &Some(
                &Tick::new(
                    U64::from(10),
                    0,
                    U64::one(),
                    true,
                    Some(U64::from(10)),
                    None,
                )
                .unwrap(),
            ),
            &Some(
                &Tick::new(
                    U64::from(10),
                    0,
                    U64::one(),
                    true,
                    Some(U64::from(10)),
                    None,
                )
                .unwrap(),
            ),
            1,
            &Tick::new(U64::from(20), 0, U64::one(), true, None, None).unwrap(),
        );
        assert!(tick.is_ok());
        let tick = tick.unwrap();
        assert!(tick.moving_average.is_some());
        assert_eq!(tick.moving_average.unwrap(), U64::from(15))
    }

    #[test]
    fn make_moving_average_empty() {
        let runner = Runner::default();
        let tick = Runner::make_sliding_moving_average(
            &runner,
            &None,
            &None,
            0,
            &Tick::new(U64::from(20), 0, U64::one(), true, None, None).unwrap(),
        );
        assert!(tick.is_ok());
        let tick = tick.unwrap();
        assert!(tick.moving_average.is_some());
        assert_eq!(tick.moving_average.unwrap(), U64::from(20))
    }

    #[test]
    fn make_moving_average_sliding() {
        let runner = Runner::new(
            U64::from(1) * U64::exp10(5),
            (15, 30_000),
            (14 * 24 * 60 * 60 * 1000, 90 * 24 * 60 * 60 * 1000),
            (U64::from(1) * U64::exp10(6), U64::from(100) * U64::exp10(6)),
            (U64::from(1) * U64::exp10(6), U64::from(100) * U64::exp10(6)),
            U64::from(1_008_000),
            2,
        )
        .unwrap();
        let tick = Runner::make_sliding_moving_average(
            &runner,
            &Some(
                &Tick::new(
                    U64::from(10),
                    0,
                    U64::one(),
                    true,
                    Some(U64::from(10)),
                    None,
                )
                .unwrap(),
            ),
            &Some(
                &Tick::new(
                    U64::from(20),
                    0,
                    U64::one(),
                    true,
                    Some(U64::from(15)),
                    None,
                )
                .unwrap(),
            ),
            2,
            &Tick::new(U64::from(30), 0, U64::one(), true, None, None).unwrap(),
        );
        assert!(tick.is_ok());
        let tick = tick.unwrap();
        assert!(tick.moving_average.is_some());
        assert_eq!(tick.moving_average.unwrap(), U64::from(25))
    }
}
