use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{calculate_tokens_to_send, errors::AMMErrors, Admin, TradeEvent, AMM};

#[derive(Accounts)]
pub struct BuyCoin<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        seeds = [b"mint", amm.creator.key().as_ref(), amm.seed.to_le_bytes().as_ref()],
        bump = amm.mint_bump,
        mint::decimals = mint.decimals,
        mint::authority = mint
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        seeds = [b"amm", mint.key().as_ref(), amm.seed.to_le_bytes().as_ref()],
        bump = amm.amm_bump,
    )]
    pub amm: Account<'info, AMM>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = amm
      )]
    pub token_reserve: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = signer
      )]
    pub signer_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
          mut,
          seeds = [b"solVault", amm.key().as_ref()],
          bump = amm.sol_reserve_bump
      )]
    pub sol_reserve: SystemAccount<'info>,
    #[account(
        mut,
        constraint = amm.creator.key() == creator.key()
    )]
    /// CHECK: This is safe
    pub creator: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"admin"],
        bump = admin.bump
    )]
    pub admin: Account<'info, Admin>,
    #[account(
        mut,
        seeds = [b"admin", admin.key().as_ref()],
        bump = admin.vault_bump
    )]
    pub admin_vault: SystemAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
impl<'info> BuyCoin<'info> {
    pub fn buy_coin(&mut self, amount: u64) -> Result<()> {
        require!(amount != 0, AMMErrors::ZeroValueError);
        require!(self.admin.is_initialized, AMMErrors::AdminNotInitalized);

        // Transfer royalty to dev and creator
        let royalty_amount = amount / 40;
        let accounts = Transfer {
            from: self.signer.to_account_info(),
            to: self.creator.to_account_info(),
        };
        let ctx = CpiContext::new(self.system_program.to_account_info(), accounts);
        transfer(ctx, royalty_amount)?;

        let accounts = Transfer {
            from: self.signer.to_account_info(),
            to: self.admin_vault.to_account_info(),
        };
        let ctx = CpiContext::new(self.system_program.to_account_info(), accounts);
        transfer(ctx, royalty_amount)?;

        let amount = amount - (royalty_amount * 2);
        let tokens_available = self.amm.mint_cap;
        let tokens_to_send = calculate_tokens_to_send(
            amount,
            self.amm.sol_cap,
            tokens_available,
            self.amm.cp_ratio,
            self.mint.decimals,
        )
        .ok()
        .unwrap();
        msg!("sol_cap: {}", self.amm.sol_cap);
        msg!("mint_cap: {}", self.amm.mint_cap);

        msg!("Swapping {}sol for {}$tokens", amount, tokens_to_send);

        let accounts = Transfer {
            from: self.signer.to_account_info(),
            to: self.sol_reserve.to_account_info(),
        };
        let ctx = CpiContext::new(self.system_program.to_account_info(), accounts);
        transfer(ctx, amount)?;

        let binding = self.amm.seed.to_le_bytes();
        let seeds = &[
            b"amm",
            self.mint.to_account_info().key.as_ref(),
            binding.as_ref(),
            &[self.amm.amm_bump],
        ];
        let signer_seeds = &[&seeds[..]];
        let accounts = TransferChecked {
            from: self.token_reserve.to_account_info(),
            mint: self.mint.to_account_info(),
            to: self.signer_ata.to_account_info(),
            authority: self.amm.to_account_info(),
        };
        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            signer_seeds,
        );
        transfer_checked(ctx, tokens_to_send, self.mint.decimals)?;

        emit!(TradeEvent {
            user: self.signer.key(),
            sol_amount: amount,
            token_amount: tokens_to_send,
            mint: self.mint.key()
        });

        self.amm.sol_cap = self.amm.sol_cap.checked_add(amount).unwrap();
        self.amm.mint_cap = self.amm.mint_cap.checked_sub(tokens_to_send).unwrap();
        msg!("new sol_cap: {}", self.amm.sol_cap);
        msg!("new mint_cap: {}", self.amm.mint_cap);
        Ok(())
    }
}
