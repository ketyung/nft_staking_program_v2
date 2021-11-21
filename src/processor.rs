use {
    solana_program::{
        account_info::{AccountInfo},
        entrypoint::ProgramResult,
        msg,
        pubkey::Pubkey,
        program_error::ProgramError,
    },
    
    crate::error::TokenProgramError,

    crate::staking_instruction::StakingInstruction, 

    crate::staking_manager::StakingManager,

};


// mod staking is 2
const MOD_STAKING : u8 = 2;

pub fn process_instruction(program_id: &Pubkey,accounts: &[AccountInfo], instruction_data: &[u8],) 
-> ProgramResult {

    
    let (module, _rest) = instruction_data.split_first().ok_or(TokenProgramError::InvalidInstruction)?;

    Ok (match module {

        &MOD_STAKING => {

            process_staking_instruction(program_id, &accounts, &_rest)?
        }
        ,

        _=> {
            msg!("Unknown module!");
        }
    })

}


fn process_staking_instruction(program_id : &Pubkey, accounts : &[AccountInfo], input : &[u8]) -> ProgramResult{


    let ins = StakingInstruction::unpack(input)?;

    match ins {

        StakingInstruction::CreateStake {for_month} => {

           // msg!("Create Stake for month {}", for_month);

            let f  = StakingManager::create_stake(program_id, &accounts, for_month);

            match f {

                Ok(_)=> { Ok(())},

                Err(error) => {

                    return Err(ProgramError::from(error));

                }
            }
        },

       
        StakingInstruction::Withdraw {token_decimal, is_simulation} => {

      
            let f  = StakingManager::withdraw(program_id, &accounts, token_decimal, is_simulation);

            match f {

                Ok(_)=> { Ok(())},

                Err(error) => {

                    return Err(ProgramError::from(error));

                }
            }
        },

        _ => {

            msg!("Program id, {:?}, accounts: {:?}", program_id, accounts);
            return Err(ProgramError::from(TokenProgramError::InvalidAction));
        }

    }

   // Ok(())
     
}