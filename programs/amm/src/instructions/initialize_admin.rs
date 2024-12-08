use crate::AMMErrors;
use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::Admin;
#[derive(Accounts)]
pub struct InitializeAdmin<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        seeds = [b"admin"],
        bump,
        space = 8 + Admin::INIT_SPACE
    )]
    pub admin: Account<'info, Admin>,
    #[account(
        mut,
        seeds = [b"admin", admin.key().as_ref()],
        bump
    )]
    pub admin_vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}
impl<'info> InitializeAdmin<'info> {
    pub fn init_admin(&mut self, bumps: &InitializeAdminBumps) -> Result<()> {
        require!(!self.admin.is_initialized, AMMErrors::AdminNotInitalized);
        let accounts = Transfer {
            from: self.signer.to_account_info(),
            to: self.admin_vault.to_account_info(),
        };
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(0);
        let ctx = CpiContext::new(self.system_program.to_account_info(), accounts);
        transfer(ctx, lamports)?;
        self.admin.set_inner(Admin {
            admin: self.signer.to_account_info().key(),
            is_initialized: true,
            bump: bumps.admin,
            vault_bump: bumps.admin_vault,
        });
        Ok(())
    }
}
