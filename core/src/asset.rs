use ethers::types::Address;

/// Asset is an ERC-20, we will work with f64 + decimal shift instead of U256 for ease of use.
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
    fn asset_new() {
        let asset = Asset::new(Address::random(), String::from("ETH"), 8);
        assert_eq!(asset.decimal_shift, 8);
    }
}
