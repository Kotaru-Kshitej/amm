use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Admin {
    pub admin: Pubkey,
    pub is_initialized: bool,
    pub bump: u8,
    pub vault_bump: u8,
}
