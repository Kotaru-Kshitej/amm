use crate::AMMErrors;
use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::Admin;
#[derive(Accounts)]
pub struct ClaimFee<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"admin"],
        bump = admin.bump,
    )]
    pub admin: Account<'info, Admin>,
    #[account(
        mut,
        seeds = [b"admin", admin.key().as_ref()],
        bump = admin.vault_bump
    )]
    pub admin_vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}
impl<'info> ClaimFee<'info> {
    pub fn claim_fee(&mut self) -> Result<()> {
        require!(self.admin.is_initialized, AMMErrors::AdminNotInitalized);
        require_keys_eq!(
            self.signer.key(),
            self.admin.admin,
            AMMErrors::NotAuthorizedError
        );

        let accounts = Transfer {
            from: self.admin_vault.to_account_info(),
            to: self.signer.to_account_info(),
        };
        let binding = &[self.admin.vault_bump];
        let binding2 = self.admin.to_account_info().key();

        let seeds: &[&[u8]] = &[b"admin", binding2.as_ref(), binding];
        let signer_seeds: &[&[&[u8]]; 1] = &[&seeds];

        let ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            accounts,
            signer_seeds,
        );
        let rent = Rent::get()?;
        let amount: u64 = self
            .admin
            .get_lamports()
            .checked_sub(rent.minimum_balance(8 + Admin::INIT_SPACE))
            .unwrap();

        msg!("Transferred fees to the owner");

        transfer(ctx, amount)?;
        Ok(())
    }
}
