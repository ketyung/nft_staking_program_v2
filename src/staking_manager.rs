use {

    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        pubkey::Pubkey,
        program_error::ProgramError,
        program_pack::{Pack},
        program::{invoke},
        clock::{Clock, UnixTimestamp},
        sysvar::Sysvar,  
    },
    crate::state::{NftStake},
    crate::index::{NftIndex},
};
use std::convert::{TryFrom};
use crate::{error::TokenProgramError};


pub struct StakingManager {}

impl StakingManager {

    pub fn create_stake(program_id: &Pubkey, accounts: &[AccountInfo], for_month : u8 )  -> ProgramResult {

        let account_info_iter = &mut accounts.iter();

        let signer_account = next_account_info(account_info_iter)?;
        
        let stake_account = next_account_info(account_info_iter)?;
    
        let nft_mint = next_account_info(account_info_iter)?;

        let token_account = next_account_info(account_info_iter)?; 

        let token_program = next_account_info(account_info_iter)?;
   
        let index_account = next_account_info(account_info_iter)?;

        if !signer_account.is_signer {

            return Err(ProgramError::MissingRequiredSignature);
        }


        if stake_account.owner != program_id {

            return Err( ProgramError::IncorrectProgramId );
        }

        if index_account.owner != program_id {

            return Err( ProgramError::IncorrectProgramId );       
        }


        // create PDA based on the NFT mint
        let addr = &[nft_mint.key.as_ref()];
        //&[b"dogecapt".as_ref()];
        
        //msg!("add.x::{:?}", addr);
       
        let (pda, _bump_seed) = Pubkey::find_program_address(addr, program_id);
       
        
        let stored_stake = NftStake::unpack_unchecked(&stake_account.data.borrow());
        
        let curr_time =  Clock::get().unwrap().unix_timestamp;

        match stored_stake {

            Ok(_) => {
    
                let mut stake = NftStake::new(curr_time);
                stake.for_month = for_month;
                stake.nft_mint = *nft_mint.key;
                stake.rate = determine_rate(for_month);
                stake.pda = pda;
                stake.owner = *signer_account.key; 

                handle_program_result (NftStake::pack(stake, &mut stake_account.data.borrow_mut()) );
    
            },
    
            Err(_) => {
    
                let mut stake = NftStake::new(curr_time);
                stake.for_month = for_month;
                stake.nft_mint = *nft_mint.key;
                stake.rate = determine_rate(for_month);
                stake.pda = pda;
                stake.owner = *signer_account.key; 
              
                handle_program_result( NftStake::pack(stake, &mut stake_account.data.borrow_mut()) );
               
            } 
            
        }
    


        
        let tf_to_pda_ix = spl_token::instruction::set_authority(
            token_program.key,
            token_account.key,
            Some(&pda), 
            spl_token::instruction::AuthorityType::AccountOwner,
            signer_account.key,
            &[&signer_account.key],
        )?;
      
        
        invoke(
            &tf_to_pda_ix,
            &[
                token_account.clone(),
                signer_account.clone(),
                token_program.clone(),
            ],
        )?;
        

       
        Self::add_to_index(*stake_account.key,  &index_account, *signer_account.key, curr_time);

        Ok(())

    }
}

impl StakingManager {

    // to be called by bot to update the reward
    pub fn update_stake(program_id: &Pubkey, accounts: &[AccountInfo])  -> ProgramResult {

        let account_info_iter = &mut accounts.iter();

        let stake_account = next_account_info(account_info_iter)?;
    
        if stake_account.owner != program_id {

            return Err( ProgramError::IncorrectProgramId );
        }

        let stored_stake = NftStake::unpack_unchecked(&stake_account.data.borrow());
   

        match stored_stake {

            Ok(mut stake) => {
    
                let curr_time = Clock::get().unwrap().unix_timestamp;

                // 6 decimals
               
                stake.last_update = curr_time;

                handle_program_result(NftStake::pack(stake, &mut stake_account.data.borrow_mut()));

            },

            Err(error) => {
    
                return Err(ProgramError::from(error));

            }

        }

        Ok(())

    }

}


impl StakingManager {

    pub fn withdraw(_program_id: &Pubkey, accounts: &[AccountInfo],
        _token_decimal : u32, _is_simulation : u8 )  -> ProgramResult{

        let account_info_iter = &mut accounts.iter();

     
        let signer_account = next_account_info(account_info_iter)?;

        if !signer_account.is_signer {

            return Err( ProgramError::from(TokenProgramError::InvalidSigner) );
        }


        let dc_token_mint = next_account_info(account_info_iter)?;

        if *dc_token_mint.owner != spl_token::id() {

            // if the token mint isn't SPL token program
            return Err(ProgramError::IncorrectProgramId);

        }

        let _tx_token_account = next_account_info(account_info_iter)?;

        let _spl_token_program = next_account_info(account_info_iter)?; 

        let mut _accumulated_token_count : u64 = 0; 

        // to continue ....


        Ok(())
    }

}


impl StakingManager {

    fn add_to_index(nft : Pubkey, index_account : &AccountInfo, owner : Pubkey, curr_time : UnixTimestamp ) {

        let index = NftIndex::unpack_unchecked(&index_account.data.borrow());
    
        
        match index{
    
            Ok(mut idx) => {
    
                if idx.len() == 0 {
                    idx.owner = owner;
                    idx.first_stake_date = curr_time;
                }

                idx.add_nft(nft);

                handle_program_result( NftIndex::pack(idx, &mut index_account.data.borrow_mut()) );
    
            },
    
            Err(_) => {
    
              
                let mut index = NftIndex::new(curr_time);

                index.owner = owner;

                index.add_nft(nft);
        
               
                handle_program_result( NftIndex::pack(index, &mut index_account.data.borrow_mut()) );
    
            }
    
        }
    
    }
    
}


// rate is per day 
fn determine_rate(for_month : u8 ) -> u64 {

    match for_month {

        1 => {

            return 5;
        }
        ,
        2 => {

            return 3;
        },
        
        0 => {

            return 10; 
        },

        _ => {

            return 2;
        }

    }

}

pub fn handle_program_result(_result : ProgramResult) {

    match _result {

        Ok(_) => { },

        Err(e) => {

            msg!("handle_program_result:failed.with.error.x:::{:?}", e);
            
        }
    }
}