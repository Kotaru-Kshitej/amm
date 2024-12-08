use anchor_lang::prelude::*;
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct InitTokenParams {
    pub name: String,
    pub seed: u64,
    pub symbol: String,
    pub uri: String,
    pub sol_cap: u64,
    pub mint_cap: u64,
    pub decimals: u8,
}
