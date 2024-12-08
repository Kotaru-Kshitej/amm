use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct AMM {
    pub mint: Pubkey,
    #[max_len(70)]
    pub uri: String,
    pub creator: Pubkey,
    pub cp_ratio: u64,
    pub sol_cap: u64,
    pub mint_cap: u64,
    pub sol_reserve_bump: u8,
    pub seed: u64,
    pub amm_bump: u8,
    pub mint_bump: u8,
}
// impl Space for AMM {
//     const INIT_SPACE: usize = 32 + (4 + 70) + 32 + 8 + 8 + 8 + 1 + 1 + 1;
// }
