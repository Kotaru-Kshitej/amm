use anchor_lang::prelude::*;

#[event]
pub struct AmmInitalized {
    pub mint: Pubkey,
    pub creator: Pubkey,
    pub cp_ratio: u64,
    pub sol_cap: u64,
    pub mint_cap: u64,
}

#[event]
pub struct TradeEvent {
    pub user: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub mint: Pubkey,
}
