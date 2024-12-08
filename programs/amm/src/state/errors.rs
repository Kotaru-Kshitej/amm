use anchor_lang::prelude::*;

#[error_code]
pub enum AMMErrors {
    #[msg("Please enter valid token quantity")]
    TooManyTokens,
    #[msg("Value cannot be zero")]
    ZeroValueError,
    #[msg("Please reduce your market cap")]
    CPRatioOverflowError,
    #[msg("Admin not initalized")]
    AdminNotInitalized,
    #[msg("You are not authorized to execute this instruction")]
    NotAuthorizedError,
}
