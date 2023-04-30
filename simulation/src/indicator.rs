use crate::market::Tick;
use crate::mul_div::mul_div_i256;
use crate::runner::Runner;
use ethers::types::{I256, U64};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum IndicatorError {
    #[error("Moving average muldiv by old len overflow ({0} muldiv {1})")]
    MovingAverageMulDivLenOverflow(I256, I256),
    #[error("Ticks len not zero so first and last tick price and moving average should be some.")]
    FirstLastTickMovingAverageNone(),
    #[error("Need the moving average of new tick to compute the variance.")]
    NewTickMovingAverageForVarianceNone(),
    #[error("Need the variance of last tick to compute the variance.")]
    LastTickVarianceNone(),
    #[error("Moving average can't be negative({0})")]
    MovingAverageIsNegative(I256),
    #[error("Need last moving average to compute new variance.")]
    NewTickNoAverageForVariance(),
    #[error("Variance muldiv by old len overflow ({0} muldiv {1})")]
    VarianceMulDivLenOverflow(I256, I256),
}

pub fn make_indicators_from_ticks(
    _runner: &Runner,
    _old_ticks: &Vec<Tick>,
    _new_ticks: &Vec<Tick>,
    _new_tick: &Tick,
) -> Result<Tick, IndicatorError> {
    enum TicksState {
        Zero,
        FirstNewLastNew,
        FirstOldLastNew,
        FirstOldLastOld,
    }
    let mut state: TicksState = TicksState::FirstOldLastNew;
    let tick_len = _runner
        .duration_moving_average_tick
        .min(_old_ticks.len() + _new_ticks.len());

    let is_ticks_zero = tick_len == 0;
    if is_ticks_zero {
        state = TicksState::Zero;
        return make_sliding_moving_average(_runner, &None, &None, tick_len, _new_tick);
    }
    if _new_ticks.is_empty() {
        state = TicksState::FirstOldLastOld;
    }
    if _old_ticks.is_empty() {
        state = TicksState::FirstNewLastNew
    }
    let is_new_full = _new_ticks.len() >= _runner.duration_moving_average_tick;
    if is_new_full {
        state = TicksState::FirstNewLastNew
    }

    return match state {
        TicksState::Zero => make_sliding_moving_average(_runner, &None, &None, tick_len, _new_tick),
        TicksState::FirstNewLastNew => make_sliding_moving_average(
            // old[...] new[1, =>2<=, 3, =>4<=] len=3
            _runner,
            &_new_ticks.get(
                0.max((_new_ticks.len() as i64) - (_runner.duration_moving_average_tick as i64))
                    as usize,
            ),
            &_new_ticks.last(),
            tick_len,
            _new_tick,
        ),
        TicksState::FirstOldLastNew => make_sliding_moving_average(
            // old[1, 2, 3, =>4<=] new [5, =>6<=] len=3
            _runner,
            &_old_ticks.get(0.max(
                (_old_ticks.len() as i64)
                    - ((_runner.duration_moving_average_tick as i64) - (_new_ticks.len() as i64)),
            ) as usize),
            &_new_ticks.last(),
            tick_len,
            _new_tick,
        ),
        TicksState::FirstOldLastOld => make_sliding_moving_average(
            // old[1, =>2<=, 3, =>4<=] new [] len=3
            _runner,
            &_old_ticks.get(
                0.max((_old_ticks.len() as i64) - (_runner.duration_moving_average_tick as i64))
                    as usize,
            ),
            &_old_ticks.last(),
            tick_len,
            _new_tick,
        ),
    };
}

pub fn make_sliding_moving_average(
    _runner: &Runner,
    _first_tick: &Option<&Tick>,
    _last_tick: &Option<&Tick>,
    _tick_len: usize,
    _new_tick: &Tick,
) -> Result<Tick, IndicatorError> {
    let is_first_last_tick_some = _first_tick.is_some()
        && _last_tick.is_some()
        && _first_tick.unwrap().moving_average.is_some()
        && _last_tick.unwrap().moving_average.is_some();
    let mut tick = _new_tick.clone();
    match _tick_len {
        i if i == 0 => {
            tick.moving_average = Some(tick.price);
        }
        i if i < _runner.duration_moving_average_tick => {
            // average_new = old_average + ((new_value - old_average)/new_size)
            if !is_first_last_tick_some {
                return Err(IndicatorError::FirstLastTickMovingAverageNone());
            }
            let old_average = I256::from(_last_tick.unwrap().moving_average.unwrap().as_u64());
            let new_value = I256::from(_new_tick.price.as_u64());
            let new_size = I256::from(_tick_len + 1) * I256::exp10(6);
            let mut moving_average = new_value - old_average;
            moving_average = old_average
                + mul_div_i256(moving_average, I256::exp10(6), new_size).ok_or(
                    IndicatorError::MovingAverageMulDivLenOverflow(moving_average, new_size),
                )?;
            tick.moving_average = Some(U64::from(moving_average.abs().as_u64()));
        }
        _ => {
            // average_new = old_average + ((new_value - removed_value)/new_size)
            if !is_first_last_tick_some {
                return Err(IndicatorError::FirstLastTickMovingAverageNone());
            }
            let old_average = I256::from(_last_tick.unwrap().moving_average.unwrap().as_u64());
            let new_value = I256::from(_new_tick.price.as_u64());
            let removed_value = I256::from(_first_tick.unwrap().price.as_u64());
            let new_size = I256::from(_tick_len) * I256::exp10(6);
            let mut moving_average = new_value - removed_value;
            moving_average = old_average
                + mul_div_i256(moving_average, I256::exp10(6), new_size).ok_or(
                    IndicatorError::MovingAverageMulDivLenOverflow(moving_average, new_size),
                )?;
            tick.moving_average = Some(U64::from(moving_average.abs().as_u64()));
        }
    }
    Ok(tick)
}

pub fn make_sliding_variance(
    _runner: &Runner,
    _first_tick: &Option<&Tick>,
    _last_tick: &Option<&Tick>,
    _tick_len: usize,
    _new_tick: &Tick,
) -> Result<Tick, IndicatorError> {
    let is_first_last_tick_some = _first_tick.is_some()
        && _last_tick.is_some()
        && _first_tick.unwrap().moving_average.is_some()
        && _last_tick.unwrap().moving_average.is_some();
    let is_new_moving_average_some = _new_tick.moving_average.is_some();
    let is_old_variance_some = _last_tick.is_some() && _last_tick.unwrap().variance.is_some();
    let mut tick = _new_tick.clone();
    match _tick_len {
        i if i == 0 => tick.variance = Some(U64::zero()),
        i if i == 1 => {
            // variance = ((new_ma - old_value)² + (new_ma - new_value)²)/new_size
            if !is_new_moving_average_some {
                return Err(IndicatorError::NewTickMovingAverageForVarianceNone());
            }
            if !is_first_last_tick_some {
                return Err(IndicatorError::FirstLastTickMovingAverageNone());
            }
            let old_value = I256::from(_last_tick.unwrap().price.as_u64());
            let new_value = I256::from(_new_tick.price.as_u64());
            let new_ma = I256::from(_new_tick.moving_average.unwrap().as_u64());
            let new_size = I256::from(2) * I256::exp10(6);
            let mut variance = (new_ma - old_value).pow(2);
            variance += (new_ma - new_value).pow(2);
            variance = mul_div_i256(variance, I256::exp10(6), new_size).ok_or(
                IndicatorError::VarianceMulDivLenOverflow(variance, new_size),
            )?;
            tick.variance = Some(U64::from(variance.abs().as_u64()));
        }
        i if i == _runner.duration_moving_average_tick => {
            // variance = old_variance + (new_ma - old_ma)²
            // + ((new_ma - new_value)² - (new_ma - removed_value)²)/new_size
            if !is_new_moving_average_some {
                return Err(IndicatorError::NewTickMovingAverageForVarianceNone());
            }
            if !is_first_last_tick_some {
                return Err(IndicatorError::FirstLastTickMovingAverageNone());
            }
            if !is_old_variance_some {
                return Err(IndicatorError::LastTickVarianceNone());
            }
            let removed_value = I256::from(_first_tick.unwrap().price.as_u64());
            let old_variance = I256::from(_last_tick.unwrap().variance.unwrap().as_u64());
            let new_value = I256::from(_new_tick.price.as_u64());
            let new_ma = I256::from(_new_tick.moving_average.unwrap().as_u64());
            let old_ma = I256::from(_last_tick.unwrap().moving_average.unwrap().as_u64());
            let new_size = I256::from(_runner.duration_moving_average_tick) * I256::exp10(6);
            let mut variance = (new_ma - new_value).pow(2);
            variance -= (new_ma - removed_value).pow(2);
            variance = mul_div_i256(variance, I256::exp10(6), new_size).ok_or(
                IndicatorError::VarianceMulDivLenOverflow(variance, new_size),
            )?;
            variance += (new_ma - old_ma).pow(2);
            variance += old_variance;
            tick.variance = Some(U64::from(variance.abs().as_u64()));
        }
        _ => {
            // variance =  (old_size / new_size) * (old_variance + ((old_ma - new_value)²/new_size))
            if !is_new_moving_average_some {
                return Err(IndicatorError::NewTickMovingAverageForVarianceNone());
            }
            if !is_first_last_tick_some {
                return Err(IndicatorError::FirstLastTickMovingAverageNone());
            }
            if !is_old_variance_some {
                return Err(IndicatorError::LastTickVarianceNone());
            }
            let old_variance = I256::from(_last_tick.unwrap().variance.unwrap().as_u64());
            let new_value = I256::from(_new_tick.price.as_u64());
            let old_ma = I256::from(_last_tick.unwrap().moving_average.unwrap().as_u64());
            let old_size = I256::from(_tick_len) * I256::exp10(6);
            let new_size = I256::from(_tick_len + 1) * I256::exp10(6);
            let mut variance = (old_ma - new_value).pow(2);
            variance = mul_div_i256(variance, I256::exp10(6), new_size).ok_or(
                IndicatorError::VarianceMulDivLenOverflow(variance, new_size),
            )?;
            variance += old_variance;
            variance = mul_div_i256(variance, old_size, new_size).ok_or(
                IndicatorError::VarianceMulDivLenOverflow(variance, new_size),
            )?;
            tick.variance = Some(U64::from(variance.abs().as_u64()));
        }
    }
    Ok(tick)
}

#[cfg(test)]
mod tests {
    use crate::indicator::{
        make_indicators_from_ticks, make_sliding_moving_average, make_sliding_variance,
    };
    use crate::market::Tick;
    use crate::runner::Runner;
    use ethers::types::U64;

    #[test]
    fn make_moving_average_success() {
        let runner = Runner::default();
        let tick = make_sliding_moving_average(
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
        assert_eq!(tick.moving_average.unwrap(), U64::from(15));

        let tick = make_sliding_moving_average(
            &runner,
            &Some(
                &Tick::new(
                    U64::from(20),
                    0,
                    U64::one(),
                    true,
                    Some(U64::from(20)),
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
                    Some(U64::from(20)),
                    None,
                )
                .unwrap(),
            ),
            1,
            &Tick::new(U64::from(10), 0, U64::one(), true, None, None).unwrap(),
        );
        assert!(tick.is_ok());
        let tick = tick.unwrap();
        assert!(tick.moving_average.is_some());
        assert_eq!(tick.moving_average.unwrap(), U64::from(15))
    }

    #[test]
    fn make_moving_average_empty() {
        let runner = Runner::default();
        let tick = make_sliding_moving_average(
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
        let mut runner = Runner::default();
        runner.duration_moving_average_tick = 2;
        let tick = make_sliding_moving_average(
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

    #[test]
    fn make_indicators_from_ticks_full_old() {
        let mut runner = Runner::default();
        runner.duration_moving_average_tick = 3;
        let old_ticks = vec![
            Tick::new(
                U64::from(10),
                0,
                U64::one(),
                true,
                Some(U64::from(10)),
                None,
            )
            .unwrap(),
            Tick::new(
                U64::from(20),
                0,
                U64::one(),
                true,
                Some(U64::from(15)),
                None,
            )
            .unwrap(),
            Tick::new(
                U64::from(30),
                0,
                U64::one(),
                true,
                Some(U64::from(20)),
                None,
            )
            .unwrap(),
        ];
        let tick = make_indicators_from_ticks(
            &runner,
            &old_ticks,
            &vec![],
            &Tick::new(U64::from(40), 0, U64::one(), true, None, None).unwrap(),
        );
        assert!(tick.is_ok());
        let tick = tick.unwrap();
        assert!(tick.moving_average.is_some());
        assert_eq!(tick.moving_average.unwrap(), U64::from(30));
    }

    #[test]
    fn make_indicators_from_ticks_full_new() {
        let mut runner = Runner::default();
        runner.duration_moving_average_tick = 3;
        let new_ticks = vec![
            Tick::new(
                U64::from(10),
                0,
                U64::one(),
                true,
                Some(U64::from(10)),
                None,
            )
            .unwrap(),
            Tick::new(
                U64::from(20),
                0,
                U64::one(),
                true,
                Some(U64::from(15)),
                None,
            )
            .unwrap(),
            Tick::new(
                U64::from(30),
                0,
                U64::one(),
                true,
                Some(U64::from(20)),
                None,
            )
            .unwrap(),
        ];
        let tick = make_indicators_from_ticks(
            &runner,
            &vec![],
            &new_ticks,
            &Tick::new(U64::from(40), 0, U64::one(), true, None, None).unwrap(),
        );
        assert!(tick.is_ok());
        let tick = tick.unwrap();
        assert!(tick.moving_average.is_some());
        assert_eq!(tick.moving_average.unwrap(), U64::from(30));
    }

    #[test]
    fn make_indicators_from_ticks_partial_old() {
        let mut runner = Runner::default();
        runner.duration_moving_average_tick = 3;
        let old_ticks = vec![
            Tick::new(
                U64::from(10),
                0,
                U64::one(),
                true,
                Some(U64::from(10)),
                None,
            )
            .unwrap(),
            Tick::new(
                U64::from(20),
                0,
                U64::one(),
                true,
                Some(U64::from(15)),
                None,
            )
            .unwrap(),
        ];
        let tick = make_indicators_from_ticks(
            &runner,
            &old_ticks,
            &vec![],
            &Tick::new(U64::from(30), 0, U64::one(), true, None, None).unwrap(),
        );
        assert!(tick.is_ok());
        let tick = tick.unwrap();
        assert!(tick.moving_average.is_some());
        assert_eq!(tick.moving_average.unwrap(), U64::from(20));
    }

    #[test]
    fn make_indicators_from_ticks_partial_new() {
        let mut runner = Runner::default();
        runner.duration_moving_average_tick = 3;
        let new_ticks = vec![
            Tick::new(
                U64::from(10),
                0,
                U64::one(),
                true,
                Some(U64::from(10)),
                None,
            )
            .unwrap(),
            Tick::new(
                U64::from(20),
                0,
                U64::one(),
                true,
                Some(U64::from(15)),
                None,
            )
            .unwrap(),
        ];
        let tick = make_indicators_from_ticks(
            &runner,
            &vec![],
            &new_ticks,
            &Tick::new(U64::from(30), 0, U64::one(), true, None, None).unwrap(),
        );
        assert!(tick.is_ok());
        let tick = tick.unwrap();
        assert!(tick.moving_average.is_some());
        assert_eq!(tick.moving_average.unwrap(), U64::from(20));
    }

    #[test]
    fn make_indicators_from_ticks_full_new_old() {
        let mut runner = Runner::default();
        runner.duration_moving_average_tick = 3;
        let old_ticks = vec![
            Tick::new(
                U64::from(10),
                0,
                U64::one(),
                true,
                Some(U64::from(10)),
                None,
            )
            .unwrap(),
            Tick::new(
                U64::from(20),
                0,
                U64::one(),
                true,
                Some(U64::from(15)),
                None,
            )
            .unwrap(),
        ];
        let new_ticks = vec![Tick::new(
            U64::from(30),
            0,
            U64::one(),
            true,
            Some(U64::from(20)),
            None,
        )
        .unwrap()];
        let tick = make_indicators_from_ticks(
            &runner,
            &old_ticks,
            &new_ticks,
            &Tick::new(U64::from(40), 0, U64::one(), true, None, None).unwrap(),
        );
        assert!(tick.is_ok());
        let tick = tick.unwrap();
        assert!(tick.moving_average.is_some());
        assert_eq!(tick.moving_average.unwrap(), U64::from(30));
    }

    #[test]
    fn make_indicators_from_ticks_partal_new_old() {
        let mut runner = Runner::default();
        runner.duration_moving_average_tick = 3;
        let old_ticks = vec![Tick::new(
            U64::from(10),
            0,
            U64::one(),
            true,
            Some(U64::from(10)),
            None,
        )
        .unwrap()];
        let new_ticks = vec![Tick::new(
            U64::from(20),
            0,
            U64::one(),
            true,
            Some(U64::from(15)),
            None,
        )
        .unwrap()];
        let tick = make_indicators_from_ticks(
            &runner,
            &old_ticks,
            &new_ticks,
            &Tick::new(U64::from(30), 0, U64::one(), true, None, None).unwrap(),
        );
        assert!(tick.is_ok());
        let tick = tick.unwrap();
        assert!(tick.moving_average.is_some());
        assert_eq!(tick.moving_average.unwrap(), U64::from(20));
    }

    #[test]
    fn make_variance_len_0() {
        let runner = Runner::default();
        let tick = make_sliding_variance(
            &runner,
            &None,
            &None,
            0,
            &Tick::new(
                U64::from(20),
                0,
                U64::one(),
                true,
                Some(U64::from(15)),
                None,
            )
            .unwrap(),
        );
        assert!(tick.is_ok());
        let tick = tick.unwrap();
        assert!(tick.variance.is_some());
        assert_eq!(tick.variance.unwrap(), U64::from(0));
    }

    #[test]
    fn make_variance_len_1() {
        let runner = Runner::default();
        let tick = make_sliding_variance(
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
            &Tick::new(
                U64::from(20),
                0,
                U64::one(),
                true,
                Some(U64::from(15)),
                None,
            )
            .unwrap(),
        );
        assert!(tick.is_ok());
        let tick = tick.unwrap();
        assert!(tick.variance.is_some());
        assert_eq!(tick.variance.unwrap(), U64::from(25));
    }

    #[test]
    fn make_variance_len_2() {
        let runner = Runner::default();
        let tick = make_sliding_variance(
            &runner,
            &Some(
                &Tick::new(
                    U64::from(10),
                    0,
                    U64::one(),
                    true,
                    Some(U64::from(10)),
                    Some(U64::from(0)),
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
                    Some(U64::from(25)),
                )
                .unwrap(),
            ),
            2,
            &Tick::new(
                U64::from(30),
                0,
                U64::one(),
                true,
                Some(U64::from(20)),
                None,
            )
            .unwrap(),
        );
        assert!(tick.is_ok());
        let tick = tick.unwrap();
        assert!(tick.variance.is_some());
        assert_eq!(tick.variance.unwrap(), U64::from(66));
    }

    #[test]
    fn make_variance_len_max() {
        let mut runner = Runner::default();
        runner.duration_moving_average_tick = 2;
        let tick = make_sliding_variance(
            &runner,
            &Some(
                &Tick::new(
                    U64::from(10),
                    0,
                    U64::one(),
                    true,
                    Some(U64::from(10)),
                    Some(U64::from(0)),
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
                    Some(U64::from(25)),
                )
                .unwrap(),
            ),
            2,
            &Tick::new(
                U64::from(40),
                0,
                U64::one(),
                true,
                Some(U64::from(30)),
                None,
            )
            .unwrap(),
        );
        assert!(tick.is_ok());
        let tick = tick.unwrap();
        assert!(tick.variance.is_some());
        assert_eq!(tick.variance.unwrap(), U64::from(100));
    }
}
