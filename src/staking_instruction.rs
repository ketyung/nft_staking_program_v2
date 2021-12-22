use crate::{error::TokenProgramError};

use solana_program::{
    msg,
    program_error::ProgramError,
};

use arrayref::{array_ref, array_refs}; 

#[derive(Clone, Debug, PartialEq)]
pub enum StakingInstruction {

    CreateStake {

        for_month : u8, 
    },

    UpdateStake,

    Withdraw{

        token_decimal : u32,

        count : u8,
        
        random_number : u8, 
    }, 


    None,
 
}

const CREATE_STAKE : u8 = 1;

const UPDATE_STAKE : u8 = 2;

const WITHDRAW : u8 = 3;

impl StakingInstruction {

    pub fn unpack(input : &[u8]) -> Result<Self, ProgramError> {

     
        let (action, rest) = input.split_first().ok_or(TokenProgramError::InvalidInstruction)?;

        Ok(match action  {

            &CREATE_STAKE => {

                let for_month = Self::unpack_staking_instruction(rest);  
                Self::CreateStake{ for_month : for_month}              

            },

            &UPDATE_STAKE => {

                Self::UpdateStake
            },

            &WITHDRAW => {

                let (token_decimal, count, random_number ) = Self::unpack_withdrawal_instruction(rest);
                Self::Withdraw{

                    token_decimal: token_decimal,
                    count : count,
                    random_number : random_number,  
                }
            },

    
            _ => {

                msg!("Unknown action!");

                Self::None 
            }

        })

    }
}


impl StakingInstruction {


    fn unpack_staking_instruction (input : &[u8]) -> u8 {

        const S : usize = 1; 

        let for_month = array_ref![input, 0, S];

        return u8::from_le_bytes(*for_month);
       
    }


    fn unpack_withdrawal_instruction (input : &[u8]) -> (u32, u8, u8) {

        const S : usize = 6; 

        let src = array_ref![input, 0, S];

        let (token_decimal, count, random_number ) = array_refs![src, 4,1,1];

        ( u32::from_le_bytes(*token_decimal), u8::from_le_bytes(*count), u8::from_le_bytes(*random_number))
       
    }
}
