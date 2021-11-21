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