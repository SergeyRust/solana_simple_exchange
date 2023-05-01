use solana_program::decode_error::DecodeError;
use solana_program::msg;
use solana_program::program_error::{PrintProgramError, ProgramError};
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum TokenError {
    #[error("Not enough balance to exchange")]
    NotEnoughBalanceToExchange,
    #[error("Could not get data feed from oracle")]
    OracleDataFeedError,
    #[error("Invalid Associated Token Account")]
    InvalidAssociatedTokenAccount,
    #[error("Mismatched accounts")]
    MismatchedAccountsError,
    #[error("Account is frozen")]
    AccountFrozen,
    #[error("The provided decimals value different from the Mint decimals")]
    MintDecimalsMismatch,
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Account not associated with this Mint")]
    MintMismatch,
    #[error("Operation overflowed")]
    Overflow,
}

impl From<TokenError> for ProgramError {
    fn from(e: TokenError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for TokenError {
    fn type_of() -> &'static str {
        "TokenError"
    }
}

impl PrintProgramError for TokenError {
    fn print<E>(&self)
        where
            E: 'static
            + std::error::Error
            + DecodeError<E>
            + PrintProgramError
            + num_traits::FromPrimitive,
    {
        match self {
            TokenError::NotEnoughBalanceToExchange => msg!("Error: Not enough balance to exchange"),
            TokenError::OracleDataFeedError => msg!("Error: Could not get data feed from oracle"),
            TokenError::InvalidAssociatedTokenAccount => msg!("Error: Invalid Associated Token Account"),
            TokenError::MismatchedAccountsError => msg!("Error: Mismatched accounts"),
            TokenError::AccountFrozen => msg!("Error: Account is frozen"),
            TokenError::MintDecimalsMismatch => msg!("The provided decimals value different from the Mint decimals"),
            TokenError::InsufficientFunds => msg!("Error: Insufficient funds"),
            TokenError::MintMismatch => msg!("Error: Account not associated with this Mint"),
            TokenError::Overflow => msg!("Error: Operation overflowed"),
        }
    }
}