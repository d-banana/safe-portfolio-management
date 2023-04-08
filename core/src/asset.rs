use ethers::types::Address;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, PartialEq)]
pub struct Asset {
    pub id: String,
    pub display_name: String,
}

/// Asset is an ERC-20, we will work with f64 + decimal shift instead of U256 for ease of use.
#[derive(Debug)]
pub struct Erc20 {
    pub asset: Asset,
    pub erc20_address: Address,
    pub decimal_shift: usize,
}

impl Asset {
    pub fn new(id: String, display_name: String) -> Self {
        Self { id, display_name }
    }
}

impl Erc20 {
    pub fn new(asset: Asset, erc20_address: Address, decimal_shift: usize) -> Self {
        Self {
            asset,
            erc20_address,
            decimal_shift,
        }
    }
}

impl Hash for Asset {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_new() {
        let asset = Asset::new(String::from("ETH"), String::from("Ether"));
        assert_eq!(asset.id, String::from("ETH"));
    }

    #[test]
    fn erc20_new() {
        let asset = Asset::new(String::from("ETH"), String::from("Ether"));
        let erc20 = Erc20::new(asset, Address::random(), 8);
        assert_eq!(erc20.decimal_shift, 8);
    }
}
