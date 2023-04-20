use crate::market::{Tick, TickError};
use crate::market_state::*;
use rand::{thread_rng, Rng};
use std::io::Error;

pub struct ProvideLiquidity {
    pub spread_tick: usize,
    pub liquidity_by_tick: f64,
    pub liquidity_rate_of_change_by_tick: f64,
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
    pub duration_millisecond: u64,
    pub price_increment_percent: f64,
    pub start_price: f64,
    pub range_duration_between_trade_millisecond: (u64, u64),
    pub range_duration_between_state_millisecond: (u64, u64),
}

impl Runner {
    pub fn new(
        duration_millisecond: u64,
        price_increment_percent: f64,
        start_price: f64,
        range_duration_between_trade_millisecond: (u64, u64),
        range_duration_between_state_millisecond: (u64, u64),
    ) -> Self {
        Self {
            duration_millisecond,
            price_increment_percent,
            start_price,
            range_duration_between_trade_millisecond,
            range_duration_between_state_millisecond,
        }
    }

    pub fn run(&self) -> Result<Vec<Tick>, Error> {
        let mut rng = thread_rng();
        let mut ticks: Vec<Tick> = Vec::new();
        let mut current_time_millisecond: u64 = 0;
        let mut current_price = self.start_price;

        while current_time_millisecond < self.duration_millisecond {
            let mut market_state = MARKET_STATE::MB_GREATER_LS_MS_LESS_LB;
            let mut duration_market_state_millisecond = rng.gen_range(
                self.range_duration_between_state_millisecond.0
                    ..=self.range_duration_between_state_millisecond.1,
            );
            duration_market_state_millisecond = if duration_market_state_millisecond
                + current_time_millisecond
                > self.duration_millisecond
            {
                self.duration_millisecond - current_time_millisecond
            } else {
                duration_market_state_millisecond
            };
            ticks.append(&mut self.process_market_state(
                &current_price,
                &market_state,
                &mut current_time_millisecond,
                &duration_market_state_millisecond,
            ));
            current_time_millisecond += duration_market_state_millisecond;
            if ticks.last().is_some() {
                current_price = ticks.last().unwrap().price;
            }
        }

        Ok(ticks)
    }

    pub fn process_market_state(
        &self,
        current_price: &f64,
        market_state: &MARKET_STATE,
        current_time_millisecond: &mut u64,
        duration_market_state_millisecond: &u64,
    ) -> Vec<Tick> {
        let mut rng = thread_rng();
        let mut ticks: Vec<Tick> = Vec::new();
        let mut duration_market_state_millisecond = duration_market_state_millisecond.clone();
        let mut current_time_millisecond = current_time_millisecond.clone();
        let mut current_price = current_price.clone();
        let actor_power = market_state.actor_power();

        while duration_market_state_millisecond > 0 {
            let mut limit_sell_volume_by_tick = match actor_power.market_buyer_vs_limit_seller {
                ACTOR_POWER_STATE::LESS => {
                    rng.gen_range(LIQUIDITY_BEST.0..=LIQUIDITY_BEST.1) * ACTOR_LIQUIDITY_AMPLIFIER
                }
                ACTOR_POWER_STATE::EQUAL => rng.gen_range(LIQUIDITY_BEST.0..=LIQUIDITY_BEST.1),
                ACTOR_POWER_STATE::GREATER => {
                    rng.gen_range(LIQUIDITY_BEST.0..=LIQUIDITY_BEST.1) / ACTOR_LIQUIDITY_AMPLIFIER
                }
            } * LIMIT_LIQUIDITY_AMPLIFIER;
            let limit_sell_volume_change_by_tick = match actor_power.market_buyer_vs_limit_seller {
                ACTOR_POWER_STATE::LESS => {
                    rng.gen_range(LIQUIDITY_CHANGE_BY_TICK.0..=LIQUIDITY_CHANGE_BY_TICK.1)
                        * ACTOR_LIQUIDITY_AMPLIFIER
                }
                ACTOR_POWER_STATE::EQUAL => {
                    rng.gen_range(LIQUIDITY_CHANGE_BY_TICK.0..=LIQUIDITY_CHANGE_BY_TICK.1)
                }
                ACTOR_POWER_STATE::GREATER => {
                    rng.gen_range(LIQUIDITY_CHANGE_BY_TICK.0..=LIQUIDITY_CHANGE_BY_TICK.1)
                        / ACTOR_LIQUIDITY_AMPLIFIER
                }
            } * LIMIT_LIQUIDITY_AMPLIFIER;

            let mut limit_buy_volume_by_tick = match actor_power.market_seller_vs_limit_buyer {
                ACTOR_POWER_STATE::LESS => {
                    rng.gen_range(LIQUIDITY_BEST.0..=LIQUIDITY_BEST.1) * ACTOR_LIQUIDITY_AMPLIFIER
                }
                ACTOR_POWER_STATE::EQUAL => rng.gen_range(LIQUIDITY_BEST.0..=LIQUIDITY_BEST.1),
                ACTOR_POWER_STATE::GREATER => {
                    rng.gen_range(LIQUIDITY_BEST.0..=LIQUIDITY_BEST.1) / ACTOR_LIQUIDITY_AMPLIFIER
                }
            } * LIMIT_LIQUIDITY_AMPLIFIER;
            let limit_buy_volume_change_by_tick = match actor_power.market_seller_vs_limit_buyer {
                ACTOR_POWER_STATE::LESS => {
                    rng.gen_range(LIQUIDITY_CHANGE_BY_TICK.0..=LIQUIDITY_CHANGE_BY_TICK.1)
                        * ACTOR_LIQUIDITY_AMPLIFIER
                }
                ACTOR_POWER_STATE::EQUAL => {
                    rng.gen_range(LIQUIDITY_CHANGE_BY_TICK.0..=LIQUIDITY_CHANGE_BY_TICK.1)
                }
                ACTOR_POWER_STATE::GREATER => {
                    rng.gen_range(LIQUIDITY_CHANGE_BY_TICK.0..=LIQUIDITY_CHANGE_BY_TICK.1)
                        / ACTOR_LIQUIDITY_AMPLIFIER
                }
            } * LIMIT_LIQUIDITY_AMPLIFIER;

            let mut market_buy_volume = match actor_power.market_buyer_vs_limit_seller {
                ACTOR_POWER_STATE::GREATER => {
                    rng.gen_range(LIQUIDITY_BEST.0..=LIQUIDITY_BEST.1) * ACTOR_LIQUIDITY_AMPLIFIER
                }
                ACTOR_POWER_STATE::EQUAL => rng.gen_range(LIQUIDITY_BEST.0..=LIQUIDITY_BEST.1),
                ACTOR_POWER_STATE::LESS => {
                    rng.gen_range(LIQUIDITY_BEST.0..=LIQUIDITY_BEST.1) / ACTOR_LIQUIDITY_AMPLIFIER
                }
            };
            let mut market_sell_volume = match actor_power.market_seller_vs_limit_buyer {
                ACTOR_POWER_STATE::GREATER => {
                    rng.gen_range(LIQUIDITY_BEST.0..=LIQUIDITY_BEST.1) * ACTOR_LIQUIDITY_AMPLIFIER
                }
                ACTOR_POWER_STATE::EQUAL => rng.gen_range(LIQUIDITY_BEST.0..=LIQUIDITY_BEST.1),
                ACTOR_POWER_STATE::LESS => {
                    rng.gen_range(LIQUIDITY_BEST.0..=LIQUIDITY_BEST.1) / ACTOR_LIQUIDITY_AMPLIFIER
                }
            };

            if rng.gen_bool(0.5) {
                ticks.append(&mut self.process_market_order(
                    &current_price,
                    &current_time_millisecond,
                    &market_buy_volume,
                    &limit_sell_volume_by_tick,
                    &limit_sell_volume_change_by_tick,
                    &true,
                ));
            } else {
                ticks.append(&mut self.process_market_order(
                    &current_price,
                    &current_time_millisecond,
                    &market_sell_volume,
                    &limit_buy_volume_by_tick,
                    &limit_buy_volume_change_by_tick,
                    &false,
                ));
            }
            if ticks.last().is_some() {
                current_price = ticks.last().unwrap().price;
            }
            let wait_millisecond = rng.gen_range(
                self.range_duration_between_trade_millisecond.0
                    ..=self.range_duration_between_trade_millisecond.1,
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
                    current_price = current_price + self.price_increment_percent;
                } else {
                    current_price = current_price - self.price_increment_percent;
                }
                limit_volume_left += limit_volume_change;
            }
        }
        ticks
    }
}
