use crate::{error::TokenProgramError};

use solana_program::{
    msg,
    program_error::ProgramError,
};

use arrayref::{array_ref, array_refs};



#[derive(Clone, Debug, PartialEq)]
pub enum TokenProgramInstruction {

  
    CreateSys ,

    Transfer {

        amount : u64,

        decimal : u32, 

    },

    AddTokenVault,

    None 

}



const ACTION_CREATE_SYS : u8  = 2;

const ACTION_TX_TOKEN : u8  = 3;

const ACTION_ADD_TOKEN_VAULT : u8 = 4;




impl TokenProgramInstruction {


    pub fn unpack(input : &[u8]) -> Result<Self, ProgramError> {

        // the first byte indicating the action

        let (action, rest) = input.split_first().ok_or(TokenProgramError::InvalidInstruction)?;

        Ok(match action  {

            &ACTION_CREATE_SYS => {


                Self::CreateSys
            
            },

            &ACTION_TX_TOKEN => {

                msg!("transfering token.....x");

                let (amount, decimal)
                = Self::unpack_token_tx_instruction(&rest);
               
                Self::Transfer {
                    amount : amount, 

                    decimal : decimal,
                }
            
            },

            &ACTION_ADD_TOKEN_VAULT => {

                Self::AddTokenVault
            },

            _ => return Err(TokenProgramError::InvalidAction.into()),


        })
    }

}



impl TokenProgramInstruction {


    fn unpack_token_tx_instruction (input : &[u8]) -> (u64, u32) {

        const SZ : usize = 12; 

        let output = array_ref![input, 0, SZ];

        let (amount, decimal) = 
        array_refs![output , 8, 4];


        (u64::from_le_bytes(*amount), u32::from_le_bytes(*decimal))

        
    }
}




pub fn unpack_bool(src: &[u8; 1]) -> bool  {
    let b = u8::from_le_bytes(*src);
    match  b {
        0 =>  false,
        1 =>  true,
        _ => {
            false // if it's not 0 or 1 , just return false 
        }
    }
}