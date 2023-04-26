use ethers::types::{U256, U64};

pub fn mul_div_u64(x: U64, mul: U64, div: U64) -> Option<U64> {
    let mul_div: u64 = U256::from(x.as_u64())
        .checked_mul(U256::from(mul.as_u64()))?
        .checked_div(U256::from(div.as_u64()))?
        .try_into()
        .ok()?;
    Some(U64::from(mul_div))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::types::U64;

    #[test]
    fn mul_div_u64_success() {
        let six = mul_div_u64(U64::from(12), U64::from(10), U64::from(20));
        assert!(six.is_some());
        let six = six.unwrap();
        assert_eq!(six, U64::from(6));
    }
}
