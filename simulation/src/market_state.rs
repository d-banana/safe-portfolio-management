/// MB = Market Buy volume
/// MS = Market Sell volume
/// LB = Limit Buy volume
/// LS = Limit Sell volume
pub enum MARKET_STATE {
    // Range
    MB_EQUAL_LS_MS_EQUAL_LB,
    MB_GREATER_LS_MS_GREATER_LB,
    MB_LESS_LS_MS_LESS_LB,
    // Bullish
    MB_GREATER_LS_MS_EQUAL_LB,
    MB_GREATER_LS_MS_LESS_LB,
    MB_EQUAL_LS_MS_LESS_LB,
    //Bearish
    MB_EQUAL_LS_MS_GREATER_LB,
    MB_LESS_LS_MS_GREATER_LB,
    MB_LESS_LS_MS_EQUAL_LB,
}
#[derive(Eq, PartialEq)]
pub enum ACTOR_POWER_STATE {
    LESS,
    EQUAL,
    GREATER,
}

pub struct ActorPower {
    pub market_buyer_vs_limit_seller: ACTOR_POWER_STATE,
    pub market_seller_vs_limit_buyer: ACTOR_POWER_STATE,
}

impl ActorPower {
    pub fn new(
        market_buyer_vs_limit_seller: ACTOR_POWER_STATE,
        market_seller_vs_limit_buyer: ACTOR_POWER_STATE,
    ) -> Self {
        Self {
            market_buyer_vs_limit_seller,
            market_seller_vs_limit_buyer,
        }
    }
}

impl MARKET_STATE {
    pub fn next_state(&self, new_market_state: MARKET_STATE) -> Vec<MARKET_STATE> {
        match &self {
            MARKET_STATE::MB_EQUAL_LS_MS_EQUAL_LB => vec![
                MARKET_STATE::MB_EQUAL_LS_MS_LESS_LB,
                MARKET_STATE::MB_EQUAL_LS_MS_GREATER_LB,
                MARKET_STATE::MB_GREATER_LS_MS_EQUAL_LB,
                MARKET_STATE::MB_LESS_LS_MS_EQUAL_LB,
            ],
            MARKET_STATE::MB_GREATER_LS_MS_GREATER_LB => vec![
                MARKET_STATE::MB_GREATER_LS_MS_EQUAL_LB,
                MARKET_STATE::MB_EQUAL_LS_MS_GREATER_LB,
            ],
            MARKET_STATE::MB_LESS_LS_MS_LESS_LB => vec![
                MARKET_STATE::MB_EQUAL_LS_MS_LESS_LB,
                MARKET_STATE::MB_LESS_LS_MS_EQUAL_LB,
            ],
            MARKET_STATE::MB_GREATER_LS_MS_EQUAL_LB => vec![
                MARKET_STATE::MB_GREATER_LS_MS_GREATER_LB,
                MARKET_STATE::MB_GREATER_LS_MS_LESS_LB,
                MARKET_STATE::MB_EQUAL_LS_MS_EQUAL_LB,
            ],
            MARKET_STATE::MB_GREATER_LS_MS_LESS_LB => vec![
                MARKET_STATE::MB_GREATER_LS_MS_EQUAL_LB,
                MARKET_STATE::MB_EQUAL_LS_MS_LESS_LB,
            ],
            MARKET_STATE::MB_EQUAL_LS_MS_LESS_LB => vec![
                MARKET_STATE::MB_GREATER_LS_MS_LESS_LB,
                MARKET_STATE::MB_LESS_LS_MS_LESS_LB,
                MARKET_STATE::MB_EQUAL_LS_MS_EQUAL_LB,
            ],
            MARKET_STATE::MB_EQUAL_LS_MS_GREATER_LB => vec![
                MARKET_STATE::MB_GREATER_LS_MS_GREATER_LB,
                MARKET_STATE::MB_LESS_LS_MS_GREATER_LB,
                MARKET_STATE::MB_EQUAL_LS_MS_EQUAL_LB,
            ],
            MARKET_STATE::MB_LESS_LS_MS_GREATER_LB => vec![
                MARKET_STATE::MB_EQUAL_LS_MS_GREATER_LB,
                MARKET_STATE::MB_LESS_LS_MS_EQUAL_LB,
            ],
            MARKET_STATE::MB_LESS_LS_MS_EQUAL_LB => vec![
                MARKET_STATE::MB_LESS_LS_MS_LESS_LB,
                MARKET_STATE::MB_LESS_LS_MS_GREATER_LB,
                MARKET_STATE::MB_EQUAL_LS_MS_EQUAL_LB,
            ],
        }
    }

    pub fn actor_power(&self) -> ActorPower {
        match &self {
            MARKET_STATE::MB_EQUAL_LS_MS_EQUAL_LB => {
                ActorPower::new(ACTOR_POWER_STATE::EQUAL, ACTOR_POWER_STATE::EQUAL)
            }
            MARKET_STATE::MB_GREATER_LS_MS_GREATER_LB => {
                ActorPower::new(ACTOR_POWER_STATE::GREATER, ACTOR_POWER_STATE::GREATER)
            }
            MARKET_STATE::MB_LESS_LS_MS_LESS_LB => {
                ActorPower::new(ACTOR_POWER_STATE::LESS, ACTOR_POWER_STATE::LESS)
            }
            MARKET_STATE::MB_GREATER_LS_MS_EQUAL_LB => {
                ActorPower::new(ACTOR_POWER_STATE::GREATER, ACTOR_POWER_STATE::EQUAL)
            }
            MARKET_STATE::MB_GREATER_LS_MS_LESS_LB => {
                ActorPower::new(ACTOR_POWER_STATE::GREATER, ACTOR_POWER_STATE::LESS)
            }
            MARKET_STATE::MB_EQUAL_LS_MS_LESS_LB => {
                ActorPower::new(ACTOR_POWER_STATE::EQUAL, ACTOR_POWER_STATE::LESS)
            }
            MARKET_STATE::MB_EQUAL_LS_MS_GREATER_LB => {
                ActorPower::new(ACTOR_POWER_STATE::EQUAL, ACTOR_POWER_STATE::GREATER)
            }
            MARKET_STATE::MB_LESS_LS_MS_GREATER_LB => {
                ActorPower::new(ACTOR_POWER_STATE::LESS, ACTOR_POWER_STATE::GREATER)
            }
            MARKET_STATE::MB_LESS_LS_MS_EQUAL_LB => {
                ActorPower::new(ACTOR_POWER_STATE::LESS, ACTOR_POWER_STATE::EQUAL)
            }
        }
    }
}
