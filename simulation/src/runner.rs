use crate::market::{Tick, TickError};
use crate::market_state::*;
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum RunnerError {
    #[error("Tick error{0}")]
    Tick(TickError),
}

/// Define a taker buyer/seller vs maker seller/buyer
/// market_volume is how much will be buy/sell
/// limit_volume_by_tick is how much the take is willing to sell/buy at best price
/// if there is not enough volume to absorb the taker volume we can move to the next tick,
/// and increase the limit_volume_by_tick with limit_volume_change_by_tick
struct Actors {
    market_volume: f64,
    limit_volume_by_tick: f64,
    limit_volume_change_by_tick: f64,
}

/// Config to run a market simulation
pub struct Runner {
    pub end_time_ms: u64,
    pub price_increment: f64,
    pub start_price: f64,
    pub duration_between_trade_range_ms: (u64, u64),
    pub duration_between_market_state_range_ms: (u64, u64),
    pub volume_base_range: (f64, f64),
    pub liquidity_change_by_tick_range: (f64, f64),
    pub actor_liquidity_amplifier: f64,
}

impl Runner {
    pub fn new(
        end_time_ms: u64,
        price_increment: f64,
        start_price: f64,
        duration_between_trade_range_ms: (u64, u64),
        duration_between_market_state_range_ms: (u64, u64),
        volume_base_range: (f64, f64),
        liquidity_change_by_tick_range: (f64, f64),
        actor_liquidity_amplifier: f64,
    ) -> Self {
        Self {
            end_time_ms,
            price_increment,
            start_price,
            duration_between_trade_range_ms,
            duration_between_market_state_range_ms,
            volume_base_range,
            liquidity_change_by_tick_range,
            actor_liquidity_amplifier,
        }
    }
    pub fn default() -> Self {
        Self {
            end_time_ms: 200 * 24 * 60 * 60 * 1000,
            price_increment: 0.1,
            start_price: 1_000.0,
            duration_between_trade_range_ms: (15, 30_000),
            duration_between_market_state_range_ms: (
                14 * 24 * 60 * 60 * 1000,
                90 * 24 * 60 * 60 * 1000,
            ),
            volume_base_range: (0.01, 1.0),
            liquidity_change_by_tick_range: (0.01, 1.0),
            actor_liquidity_amplifier: 1.005,
        }
    }
    pub fn run(
        &mut self,
        mut current_time_ms: u64,
        mut current_price: f64,
    ) -> Result<Vec<Tick>, RunnerError> {
        let mut rng = thread_rng();
        let mut ticks: Vec<Tick> = Vec::new();

        while current_time_ms < self.end_time_ms {
            let current_market_state = MARKET_STATE::MB_EQUAL_LS_MS_EQUAL_LB;
            let current_duration_market_state_ms = rng
                .gen_range(
                    self.duration_between_market_state_range_ms.0
                        ..=self.duration_between_market_state_range_ms.1,
                )
                .min(self.end_time_ms - current_time_ms);
            ticks.append(&mut make_ticks_for_market_state(
                self,
                &mut rng,
                current_time_ms,
                current_price,
                current_duration_market_state_ms,
                &current_market_state,
            )?);

            current_time_ms += current_duration_market_state_ms;
            if let Some(last) = ticks.last() {
                current_price = last.price;
            }
        }

        Ok(ticks)
    }
}

fn make_ticks_for_market_state(
    _runner: &Runner,
    _rng: &mut ThreadRng,
    _current_time_ms: u64,
    _current_price: f64,
    _current_duration_market_state_ms: u64,
    _current_market_state: &MARKET_STATE,
) -> Result<Vec<Tick>, RunnerError> {
    let mut ticks: Vec<Tick> = Vec::new();
    let end_time_market_state_ms = _current_time_ms + _current_duration_market_state_ms;
    let mut current_time_market_state_ms = _current_time_ms;
    let mut current_price = _current_price;

    while current_time_market_state_ms < end_time_market_state_ms {
        let is_buy = _rng.gen_bool(0.5);
        let actors = make_actors(_runner, _rng, _current_market_state, is_buy);

        ticks.append(&mut make_ticks_for_actors(
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
            _runner.duration_between_trade_range_ms.0..=_runner.duration_between_trade_range_ms.1,
        );
    }

    Ok(ticks)
}

fn make_ticks_for_actors(
    _runner: &Runner,
    _actors: &Actors,
    _current_time_ms: u64,
    _current_price: f64,
    is_buy: bool,
) -> Result<Vec<Tick>, RunnerError> {
    let mut ticks: Vec<Tick> = Vec::new();
    let mut market_volume_left = _actors.market_volume;
    let mut limit_volume_left = _actors.limit_volume_by_tick;
    let mut current_price = _current_price;

    while market_volume_left > 0.0 && current_price > _runner.price_increment {
        let is_liquidity_consumed = market_volume_left > limit_volume_left;
        let volume = market_volume_left.min(limit_volume_left);
        market_volume_left -= volume;

        ticks.push(
            Tick::new(current_price, _current_time_ms as i64, volume, is_buy)
                .map_err(|e| RunnerError::Tick(e))?,
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

fn make_actors(
    _runner: &Runner,
    _rng: &mut ThreadRng,
    _current_market_state: &MARKET_STATE,
    _is_buy: bool,
) -> Actors {
    let actor_power = if _is_buy {
        _current_market_state
            .actor_power()
            .market_buyer_vs_limit_seller
    } else {
        _current_market_state
            .actor_power()
            .market_seller_vs_limit_buyer
    };

    let mut actors = Actors {
        market_volume: _rng.gen_range(_runner.volume_base_range.0..=_runner.volume_base_range.1),
        limit_volume_by_tick: _rng
            .gen_range(_runner.volume_base_range.0..=_runner.volume_base_range.1),
        limit_volume_change_by_tick: _rng.gen_range(
            _runner.liquidity_change_by_tick_range.0..=_runner.liquidity_change_by_tick_range.1,
        ),
    };

    match actor_power {
        ACTOR_POWER_STATE::LESS => {
            actors.market_volume /= _runner.actor_liquidity_amplifier;
            actors.limit_volume_by_tick *= _runner.actor_liquidity_amplifier;
            actors.limit_volume_change_by_tick *= _runner.actor_liquidity_amplifier;
        }
        ACTOR_POWER_STATE::EQUAL => {}
        ACTOR_POWER_STATE::GREATER => {
            actors.market_volume *= _runner.actor_liquidity_amplifier;
            actors.limit_volume_by_tick /= _runner.actor_liquidity_amplifier;
            actors.limit_volume_change_by_tick /= _runner.actor_liquidity_amplifier;
        }
    }
    actors
}
