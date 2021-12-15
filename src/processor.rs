use {
    solana_program::{
        account_info::{AccountInfo},
        entrypoint::ProgramResult,
        msg,
        pubkey::Pubkey,
        program_error::ProgramError,
    },
    crate::error::TokenProgramError,
    crate::token_instruction::TokenProgramInstruction, 
    crate::staking_instruction::StakingInstruction, 
    crate::token_manager::TokenManager,
    crate::staking_manager::StakingManager,
    crate::vault_manager::VaultManager,
};


const MOD_TOKEN : u8 = 1;

const MOD_STAKING : u8 = 2;



pub fn process_instruction(program_id: &Pubkey,accounts: &[AccountInfo], instruction_data: &[u8],) 
-> ProgramResult {

    
    let (module, _rest) = instruction_data.split_first().ok_or(TokenProgramError::InvalidInstruction)?;

    Ok (match module {

        &MOD_TOKEN => {

            process_token_instruction(program_id, &accounts, &_rest)?

        },

        &MOD_STAKING => {

            process_staking_instruction(program_id, &accounts, &_rest)?
        }
        ,

        _=> {
            msg!("Unknown module");
        }
    })

}


fn process_token_instruction (program_id : &Pubkey, accounts : &[AccountInfo], input : &[u8]) -> ProgramResult{


    let ins = TokenProgramInstruction::unpack(input)?;
    
    match ins {

        TokenProgramInstruction::CreateSys => {

          
            TokenManager::create_sys(program_id, &accounts)?;

        },

        TokenProgramInstruction::Transfer{amount, decimal} => {

            let _ = TokenManager::transfer_token_to_sys(program_id, &accounts, amount,decimal);
        },

        TokenProgramInstruction::AddTokenVault => {

            let _ = VaultManager::add_vault(program_id, &accounts);
        },

        _ => {

            msg!("Program id, {:?}, accounts: {:?}", program_id, accounts);
        }
        
    }

    Ok(())
   
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

       
        StakingInstruction::Withdraw {token_decimal, count, random_number} => {

      
            let f  = StakingManager::withdraw(program_id, &accounts, token_decimal, count, random_number);

            match f {

                Ok(_)=> { Ok(())},

                Err(error) => {

                    return Err(ProgramError::from(error));

                }
            }
        },

        StakingInstruction::Unstake => {

            let mut accumulated_token_count : u64 = 0;

            // 5th parameter random number is always 0
            // for individual unstaking so, there is NO burning of NFT
            // 6th parameter is always false as this is NOT withdrawal
            let f = StakingManager::unstake_account(program_id, &accounts, &mut accumulated_token_count, 
                crate::state::DC_TOKEN_DECIMAL, 0, false);  

            match f {

                Ok(_)=> { Ok(())},

                Err(error) => {

                    return Err(ProgramError::from(error));

                }
            }
        }, 


        StakingInstruction::Restake {for_month} => {

            // msg!("Create Stake for month {}", for_month);
 
            let f  = StakingManager::restake(program_id, &accounts, for_month);
 
           // msg!("returned.rwrd::{}", stake_rwd);

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