use anchor_lang::prelude::*;

declare_id!("8p1pfWRjJkLeonLGuU2duiT3vrqvn4SK5QEeYQ4haoyY");
pub mod events;
pub mod instructions;
pub mod state;
pub mod utils;

pub use events::*;
pub use instructions::*;
pub use state::*;
pub use utils::*;

#[program]
pub mod amm {
    use super::*;

    pub fn initialize_admin(ctx: Context<InitializeAdmin>) -> Result<()> {
        ctx.accounts.init_admin(&ctx.bumps)?;
        msg!("Initalized amm successfully");
        Ok(())
    }
    pub fn change_admin(ctx: Context<ChangeAdmin>) -> Result<()> {
        ctx.accounts.change_admin()?;
        Ok(())
    }
    pub fn claim_fee(ctx: Context<ClaimFee>) -> Result<()> {
        ctx.accounts.claim_fee()?;
        Ok(())
    }
    pub fn initialize_amm(ctx: Context<InitalizeAmm>, metadata: InitTokenParams) -> Result<()> {
        ctx.accounts.create_mint(&metadata, &ctx.bumps)?;
        ctx.accounts.init_amm_state(&metadata, &ctx.bumps)?;
        ctx.accounts.mint_to_reserve(&metadata, &ctx.bumps)?;
        msg!("Initalized amm successfully");
        Ok(())
    }
    pub fn buy_coin(ctx: Context<BuyCoin>, amount: u64) -> Result<()> {
        ctx.accounts.buy_coin(amount)?;
        msg!("Bought coin successfully");
        Ok(())
    }
    pub fn sell_coin(ctx: Context<SellCoin>, token_amount: u64) -> Result<()> {
        ctx.accounts.sell_coin(token_amount)?;
        msg!("Sold coin successfully");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
