use crate::AMMErrors;
use anchor_lang::prelude::*;

use crate::Admin;
#[derive(Accounts)]
pub struct ChangeAdmin<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    /// CHECK: This is the new admin account
    pub new_admin: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"admin"],
        bump = admin.bump,
    )]
    pub admin: Account<'info, Admin>,
    pub system_program: Program<'info, System>,
}
impl<'info> ChangeAdmin<'info> {
    pub fn change_admin(&mut self) -> Result<()> {
        require!(!self.admin.is_initialized, AMMErrors::AdminNotInitalized);
        require_keys_eq!(
            self.signer.key(),
            self.admin.admin,
            AMMErrors::NotAuthorizedError
        );
        self.admin.admin = self.new_admin.to_account_info().key();
        msg!("Changed the admin");
        Ok(())
    }
}
