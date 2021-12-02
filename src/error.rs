use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;
use num_derive::{FromPrimitive};

/// Errors that may be returned by the program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum TokenProgramError {

    #[error("Invalid instruction")]
    InvalidInstruction,

    #[error("Invalid action")]
    InvalidAction,

    #[error("Invalid owner of stake")]
    InvalidOwner,

    #[error("Invalid signer")]
    InvalidSigner,

    #[error("Vault already exists")]
    VaultAlreadyExists,

    #[error("Error, already staked!")]
    AlreadyStakedError,

    #[error("Invalid mint authority!")]
    InvalidMintAuthority,

    #[error("Error, already withdrawn!")]
    AlreadyWithdrawn,

    #[error("Invalid Treasury Account!")]
    InvalidTreasuryAccount,
   
    #[error("Invalid Token Vault File Wallet Account!")]
    InvalidTokenVaultFileWallet,

    #[error("Max Stake Has Reached!")]
    MaxStakeHasReached,


}


impl From<TokenProgramError> for ProgramError {
    fn from(e: TokenProgramError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for TokenProgramError {
    fn type_of() -> &'static str {

        "TokenProgramError"
    }
}