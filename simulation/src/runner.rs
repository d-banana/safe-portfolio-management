use crate::market::{Tick, TickError};
use crate::market_state::*;
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use std::io::Error;

struct ProvideLiquidity {
    pub spread_tick: usize,
    pub liquidity_by_tick: f64,
    pub liquidity_rate_of_change_by_tick: f64,
}

struct Actors {
    market_volume: f64,
    limit_volume_by_tick: f64,
    limit_volume_change_by_tick: f64,
}

impl ProvideLiquidity {
    pub fn new(
        spread_tick: usize,
        liquidity_by_tick: f64,
        liquidity_rate_of_change_by_tick: f64,
    ) -> Self {
        Self {
            spread_tick,
            liquidity_by_tick,
            liquidity_rate_of_change_by_tick,
        }
    }
}

const LIQUIDITY_BEST: (f64, f64) = (0.01, 1.0);
const LIQUIDITY_CHANGE_BY_TICK: (f64, f64) = (0.01, 1.0);
const ACTOR_LIQUIDITY_AMPLIFIER: f64 = 1.005;
const LIMIT_LIQUIDITY_AMPLIFIER: f64 = 1.0;
pub struct Runner {
    pub simulation_duration_ms: u64,
    pub price_increment: f64,
    pub start_price: f64,
    pub range_duration_between_trade_ms: (u64, u64),
    pub range_duration_between_state_ms: (u64, u64),
}

impl Runner {
    pub fn new(
        simulation_duration_ms: u64,
        price_increment: f64,
        start_price: f64,
        range_duration_between_trade_ms: (u64, u64),
        range_duration_between_state_ms: (u64, u64),
    ) -> Self {
        Self {
            simulation_duration_ms,
            price_increment,
            start_price,
            range_duration_between_trade_ms,
            range_duration_between_state_ms,
        }
    }

    pub fn run(&self) -> Result<Vec<Tick>, Error> {
        let mut rng = thread_rng();
        let mut ticks: Vec<Tick> = Vec::new();
        let mut current_time_ms: u64 = 0;
        let mut current_price = self.start_price;

        while current_time_ms < self.simulation_duration_ms {
            let mut market_state = MARKET_STATE::MB_EQUAL_LS_MS_GREATER_LB;

            let mut duration_market_state_millisecond = random_duration_market_state(
                &mut rng,
                self.range_duration_between_state_ms.0,
                self.range_duration_between_state_ms.1,
                current_time_ms,
                self.simulation_duration_ms,
            );
            ticks.append(&mut self.process_market_state(
                current_price,
                &market_state,
                &mut current_time_ms,
                &duration_market_state_millisecond,
            ));

            current_time_ms += duration_market_state_millisecond;
            if let Some(last) = ticks.last() {
                current_price = last.price;
            }
        }

        Ok(ticks)
    }

    pub fn process_market_state(
        &self,
        current_price: f64,
        market_state: &MARKET_STATE,
        current_time_millisecond: &mut u64,
        duration_market_state_millisecond: &u64,
    ) -> Vec<Tick> {
        let mut rng = thread_rng();
        let mut ticks: Vec<Tick> = Vec::new();
        let mut duration_market_state_millisecond = duration_market_state_millisecond.clone();
        let mut current_time_millisecond = current_time_millisecond.clone();
        let mut current_price = current_price;

        while duration_market_state_millisecond > 0 {
            let is_buy = rng.gen_bool(0.5);
            let actors = generate_actors(
                &mut rng,
                market_state,
                LIQUIDITY_BEST,
                LIQUIDITY_CHANGE_BY_TICK,
                ACTOR_LIQUIDITY_AMPLIFIER,
                is_buy,
            );

            ticks.append(&mut self.process_market_order(
                &current_price,
                &current_time_millisecond,
                &actors.market_volume,
                &actors.limit_volume_by_tick,
                &actors.limit_volume_change_by_tick,
                &is_buy,
            ));

            if ticks.last().is_some() {
                current_price = ticks.last().unwrap().price;
            }
            let wait_millisecond = rng.gen_range(
                self.range_duration_between_trade_ms.0..=self.range_duration_between_trade_ms.1,
            );
            current_time_millisecond += wait_millisecond;
            duration_market_state_millisecond -=
                if wait_millisecond > duration_market_state_millisecond {
                    duration_market_state_millisecond
                } else {
                    wait_millisecond
                };
        }

        ticks
    }

    pub fn process_market_order(
        &self,
        current_price: &f64,
        current_time_millisecond: &u64,
        market_volume: &f64,
        limit_volume: &f64,
        limit_volume_change: &f64,
        is_buy: &bool,
    ) -> Vec<Tick> {
        let mut ticks: Vec<Tick> = Vec::new();
        let mut market_volume_left = market_volume.clone();
        let mut limit_volume_left = limit_volume.clone();
        let mut current_price = current_price.clone();

        while market_volume_left > 0.0 {
            let is_liquidity_consumed = market_volume_left > limit_volume_left;
            let volume = if !is_liquidity_consumed {
                market_volume_left
            } else {
                limit_volume_left
            };
            market_volume_left -= volume;

            ticks.push(
                Tick::new(
                    current_price,
                    *current_time_millisecond as i64,
                    volume,
                    *is_buy,
                )
                .unwrap(),
            );
            if is_liquidity_consumed {
                if *is_buy {
                    current_price = current_price + self.price_increment;
                } else {
                    current_price = current_price - self.price_increment;
                }
                limit_volume_left += limit_volume_change;
            }
        }
        ticks
    }
}

fn random_duration_market_state(
    rng: &mut ThreadRng,
    low: u64,
    high: u64,
    current_time_ms: u64,
    simulation_duration_ms: u64,
) -> u64 {
    let duration_market_state_ms = rng.gen_range(low..=high);
    let is_duration_too_long = duration_market_state_ms + current_time_ms > simulation_duration_ms;

    if is_duration_too_long {
        return simulation_duration_ms - current_time_ms;
    }
    duration_market_state_ms
}

fn generate_actors(
    rng: &mut ThreadRng,
    market_state: &MARKET_STATE,
    base_liquidity_range: (f64, f64),
    base_liquidity_change_range: (f64, f64),
    actor_liquidity_amplifier: f64,
    is_buy: bool,
) -> Actors {
    let actor_power = if is_buy {
        market_state.actor_power().market_buyer_vs_limit_seller
    } else {
        market_state.actor_power().market_seller_vs_limit_buyer
    };

    let mut actors = Actors {
        market_volume: rng.gen_range(base_liquidity_range.0..=base_liquidity_range.1),
        limit_volume_by_tick: rng.gen_range(base_liquidity_range.0..=base_liquidity_range.1),
        limit_volume_change_by_tick: rng
            .gen_range(base_liquidity_change_range.0..=base_liquidity_change_range.1),
    };

    match actor_power {
        ACTOR_POWER_STATE::LESS => {
            actors.market_volume /= actor_liquidity_amplifier;
            actors.limit_volume_by_tick *= actor_liquidity_amplifier;
            actors.limit_volume_change_by_tick *= actor_liquidity_amplifier;
        }
        ACTOR_POWER_STATE::EQUAL => {}
        ACTOR_POWER_STATE::GREATER => {
            actors.market_volume *= actor_liquidity_amplifier;
            actors.limit_volume_by_tick /= actor_liquidity_amplifier;
            actors.limit_volume_change_by_tick /= actor_liquidity_amplifier;
        }
    }
    actors
}
