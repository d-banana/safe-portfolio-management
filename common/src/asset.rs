use ethers::types::Address;

pub struct Asset {
    erc20_address: Address,
    display_name: String,
    decimal_shift: usize,
}

impl Asset {
    pub fn new(erc20_address: Address, display_name: String, decimal_shift: usize) -> Self {
        Self {
            erc20_address,
            display_name,
            decimal_shift,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_constructor() {
        let asset = Asset::new(Address::random(), String::from("ETH"), 8);
        assert_eq!(asset.decimal_shift, 8);
    }
}
