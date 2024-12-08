use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{calculate_sol_to_send, errors::AMMErrors, Admin, TradeEvent, AMM};

#[derive(Accounts)]
pub struct SellCoin<'info> {
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

impl<'info> SellCoin<'info> {
    pub fn sell_coin(&mut self, token_amount: u64) -> Result<()> {
        require!(token_amount != 0, AMMErrors::ZeroValueError);
        require!(token_amount < self.amm.mint_cap, AMMErrors::TooManyTokens);
        require!(self.admin.is_initialized, AMMErrors::AdminNotInitalized);

        let sol_to_send = calculate_sol_to_send(
            self.amm.sol_cap,
            self.amm.mint_cap,
            token_amount,
            self.amm.cp_ratio,
            self.mint.decimals,
        )
        .ok()
        .unwrap();

        let royalty_amount = sol_to_send / 40;
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

        let sol_to_send = sol_to_send - (royalty_amount * 2);
        msg!("Swapping {}$tokens for {}sol", token_amount, sol_to_send);

        // Send Tokens to token reserve and SOL to the payer
        let accounts = TransferChecked {
            from: self.signer_ata.to_account_info(),
            mint: self.mint.to_account_info(),
            to: self.token_reserve.to_account_info(),
            authority: self.signer.to_account_info(),
        };
        let ctx = CpiContext::new(self.token_program.to_account_info(), accounts);

        transfer_checked(ctx, token_amount, self.mint.decimals)?;

        // Send Sol to the payer
        let accounts = Transfer {
            from: self.sol_reserve.to_account_info(),
            to: self.signer.to_account_info(),
        };

        let seeds = &[
            b"solVault",
            self.amm.to_account_info().key.as_ref(),
            &[self.amm.sol_reserve_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            accounts,
            signer_seeds,
        );

        transfer(ctx, sol_to_send)?;

        emit!(TradeEvent {
            user: self.signer.key(),
            sol_amount: sol_to_send,
            token_amount: token_amount,
            mint: self.mint.key()
        });

        self.amm.sol_cap = self.amm.sol_cap.checked_sub(sol_to_send).unwrap();
        self.amm.mint_cap = self.amm.mint_cap.checked_add(token_amount).unwrap();
        msg!("new sol_cap: {}", self.amm.sol_cap);
        msg!("new mint_cap: {}", self.amm.mint_cap);
        Ok(())
    }
}
