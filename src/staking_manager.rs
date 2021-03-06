use {

    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        pubkey::Pubkey,
        program_error::ProgramError,
        program_pack::{Pack},
        program::{invoke, invoke_signed},
        clock::{Clock, UnixTimestamp},
        sysvar::Sysvar,  
        
    },
    crate::state::{NftStake},
    crate::index::{NftIndex},
 
};

use std::convert::{TryFrom};
use crate::{error::TokenProgramError};
use std::str::FromStr;

pub struct StakingManager {}

impl StakingManager {

    /*
     * The function to create the NFT stake on chain
     */
    pub fn create_stake(program_id: &Pubkey, accounts: &[AccountInfo], for_month : u8 )  -> ProgramResult {

        let account_info_iter = &mut accounts.iter();

        let signer_account = next_account_info(account_info_iter)?;
        
        let stake_account = next_account_info(account_info_iter)?;
    
        let nft_mint = next_account_info(account_info_iter)?;

        let token_account = next_account_info(account_info_iter)?; 

        let vault_token_account = next_account_info(account_info_iter)?;
        
        let token_program = next_account_info(account_info_iter)?;
   
        let index_account = next_account_info(account_info_iter)?;

        let meta_account = next_account_info(account_info_iter)?;

        let vt_file_wallet_account = next_account_info(account_info_iter)?;

        let system_program = next_account_info(account_info_iter)?;

        if !signer_account.is_signer {

            return Err(ProgramError::MissingRequiredSignature);
        }


        if stake_account.owner != program_id {

            return Err( ProgramError::IncorrectProgramId );
        }

        if index_account.owner != program_id {

            return Err( ProgramError::IncorrectProgramId );       
        }


        // This is a check of the file wallet 
        // matches what is hard-coded in Rust smart contract in crate::state::NFT_TOKEN_VALU_FILE_WALLET
       
        if *vt_file_wallet_account.key !=  Pubkey::from_str(crate::state::NFT_TOKEN_VAULT_FILE_WALLET).unwrap() {

            return Err(ProgramError::from(TokenProgramError::InvalidTokenVaultFileWallet));
        }

        // create PDA based on the NFT mint
        let addr = &[nft_mint.key.as_ref()];
        
        let (pda, _bump_seed) = Pubkey::find_program_address(addr, program_id);
       
        
        let stored_stake = NftStake::unpack_unchecked(&stake_account.data.borrow());
        
        let curr_time =  Clock::get().unwrap().unix_timestamp;

        // record the NFT stake on-chain 
        match stored_stake {

            Ok(_) => {
    
                let mut stake = NftStake::new(curr_time);
                stake.for_month = for_month;
                stake.nft_mint = *nft_mint.key;
                stake.nft_token_account = *token_account.key;
                
                stake.rate = determine_rate(for_month);
                stake.pda = pda;
                stake.owner = *signer_account.key; 
                stake.meta_key = *meta_account.key; 
                stake.vault_account = *vault_token_account.key;
               
                handle_program_result (NftStake::pack(stake, &mut stake_account.data.borrow_mut()) );
    
            },
    
            Err(_) => {
    
                let mut stake = NftStake::new(curr_time);
                stake.for_month = for_month;
                stake.nft_mint = *nft_mint.key;
                stake.nft_token_account = *token_account.key;
                
                stake.rate = determine_rate(for_month);
                stake.pda = pda;
                stake.owner = *signer_account.key; 
                stake.meta_key = *meta_account.key; 
                stake.vault_account = *vault_token_account.key;
              
                handle_program_result( NftStake::pack(stake, &mut stake_account.data.borrow_mut()) );
               
            } 
            
        }
    

        /*
        Instruction to transfer the token to the token vault account
        */
        let tf_ix = spl_token::instruction::transfer(
            token_program.key,
            token_account.key,
            vault_token_account.key,
            &signer_account.key,
            &[&signer_account.key],
            1,
        )?;
    
        // the number of accounts involved must all 
        // passed in the 2nd param array
        invoke(
            &tf_ix,
            &[
                token_account.clone(),
                signer_account.clone(),
                vault_token_account.clone(),
                token_program.clone(),
            ],
        )?;


        /* 
        Transfer the amount in lamports equivalent to 0.0036 SOL
        back to the file wallet of the token vault API
        1 SOL = 1000000000 lampports
        */
        let amount_in_lamports = 3600000;
        
       
        invoke(
            &solana_program::system_instruction::transfer(
            signer_account.key, &vt_file_wallet_account.key, amount_in_lamports),
            &[
                signer_account.clone(),
                vt_file_wallet_account.clone(),
                system_program.clone(),
            ]       
        )?;
      
       
        Self::add_to_index(*stake_account.key,  &index_account, *signer_account.key, curr_time)?;

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

                let time_diff = (curr_time - stake.stake_date)/60/1000;

                let rate = determine_rate(stake.for_month);

                // 6 decimals
                stake.token_reward = ((( u64::try_from(time_diff).unwrap() * rate) / 86400) * 1000_000) as u64;

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

    pub fn restake (program_id: &Pubkey, accounts: &[AccountInfo], for_month : u8) -> ProgramResult{


        let account_info_iter = &mut accounts.iter();

        let signer_account = next_account_info(account_info_iter)?;
        
        let stake_account = next_account_info(account_info_iter)?;
    
       // let nft_mint = next_account_info(account_info_iter)?;

        let token_account = next_account_info(account_info_iter)?; 

        let vault_token_account = next_account_info(account_info_iter)?;
        


        if !signer_account.is_signer {

            return Err(ProgramError::MissingRequiredSignature);
        }


        if stake_account.owner != program_id {

            return Err( ProgramError::IncorrectProgramId );
        }

       // Self::verify_nft_authority(&nft_mint)?;

        let token_program = next_account_info(account_info_iter)?;

        let curr_time =  Clock::get().unwrap().unix_timestamp;

        let stored_stake = NftStake::unpack_unchecked(&stake_account.data.borrow());
        
        match stored_stake {

            Ok(mut stake) => {
    
                if stake.stat == 1 {

                    assert_eq!(for_month, stake.for_month);

                    
                    //msg!("to return reward::{}", stake_reward);

                    stake.stake_date = curr_time;
                    stake.stat = 0; 
                    stake.last_update = curr_time;
                    

                    handle_program_result (NftStake::pack(stake, &mut stake_account.data.borrow_mut()) );
             
                }
                else {

                    return Err( ProgramError::from(TokenProgramError::AlreadyStakedError) );
                }
              
            },

            Err(e) =>{

                return Err(ProgramError::from(e));
            }
        }

        let tf_ix = spl_token::instruction::transfer(
            token_program.key,
            token_account.key,
            vault_token_account.key,
            &signer_account.key,
            &[&signer_account.key],
            1,
        )?;
    
        // the number of accounts involved must all 
        // passed in the 2nd param array
        invoke(
            &tf_ix,
            &[
                token_account.clone(),
                signer_account.clone(),
                vault_token_account.clone(),
                token_program.clone(),
            ],
        )?;

        Ok(())
         

    }
}



impl StakingManager {

    pub fn withdraw(program_id: &Pubkey, accounts: &[AccountInfo],
        _token_decimal : u32, count : u8, random_number : u8 )  -> ProgramResult{

        let account_info_iter = &mut accounts.iter();
        let signer_account = next_account_info(account_info_iter)?;
        let dc_token_mint = next_account_info(account_info_iter)?;
        let dc_token_account = next_account_info(account_info_iter)?;
        let dc_pda_account = next_account_info(account_info_iter)?;
        let signer_token_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?; 
        let index_account = next_account_info(account_info_iter)?;
      
        let token_decimal = crate::state::DC_TOKEN_DECIMAL;

        if !signer_account.is_signer {

            return Err( ProgramError::from(TokenProgramError::InvalidSigner) );
        }


        if *dc_token_mint.owner != spl_token::id() {

            // if the token mint isn't SPL token program
            return Err(ProgramError::IncorrectProgramId);

        }

        if index_account.owner != program_id{

            return Err(ProgramError::IncorrectProgramId);

        }
         

        let mut accumulated_token_count : u64 = 0; 

        // instructions for unstaking all the 7 staked NFTs
        // the instruction will burn the monthly airdropped NFT during unstaking

            let nft_token_account = next_account_info(account_info_iter)?;
            let vault_token_account = next_account_info(account_info_iter)?;
            let nft_pda_account = next_account_info(account_info_iter)?;
            let nft_mint_account = next_account_info(account_info_iter)?;
    
        for _n in 0..count {

            let s_stake_account = next_account_info(account_info_iter)?;

            if s_stake_account.owner != program_id{

                return Err(ProgramError::IncorrectProgramId);
            }

            let accs = &[token_program.clone(),
            nft_token_account.clone(),
            vault_token_account.clone(), nft_pda_account.clone(), 
            nft_mint_account.clone(),
            s_stake_account.clone(),
            ];
            
            handle_program_result( Self::unstake_account(&program_id, accs, &mut accumulated_token_count, 
                token_decimal, random_number, true,_n) );  

        }

        // accumulated_token_count already includes 
        // the token decimal, so there is NO need to multiply 
        // token decimal
        handle_program_result( Self::transfer_dc_token(&program_id, 
            &[token_program.clone(),  dc_token_account.clone(), 
            signer_token_account.clone(), dc_pda_account.clone()],
            accumulated_token_count ) );
        
    
        //  Self::update_index_as_withdrawn(&index_account)?;

        Ok(())
    }
}




const REWARD_BASED_ON_TIME : u64 = 86400; 

impl StakingManager{

     /**
      * The function to unstake each which is called by the withdraw function
      */
     pub fn unstake_account (program_id: &Pubkey, accounts: &[AccountInfo],
        accumulated_token_count : &mut u64, token_decimal : u32, to_burn_random : u8, 
        is_withdrawal : bool,index : u8) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();

        let token_program = next_account_info(account_info_iter)?;
        let nft_token_account = next_account_info(account_info_iter)?;
        let vault_token_account = next_account_info(account_info_iter)?;
        let pda_account = next_account_info(account_info_iter)?;
        let nft_mint_account = next_account_info(account_info_iter)?;
   
        let stake_account = next_account_info(account_info_iter)?;    
       
        if stake_account.owner == program_id {

            let stored_stake = NftStake::unpack_unchecked(&stake_account.data.borrow());
   
            match stored_stake {

                Ok(mut stake) => {
        
                    let curr_time = Clock::get().unwrap().unix_timestamp;

                    let rate = determine_rate(stake.for_month);

                    
                    if  stake.stat == 0 {

                        let time_diff = curr_time - stake.stake_date;
                    
                        let ten : u64 = 10;
    
                        let decimal = ten.pow(token_decimal);
        
                        let time_diff_earned : u64 = u64::try_from(time_diff).unwrap() * rate;
    
                        let ratio = (StakingManager::convert( time_diff_earned).unwrap_or(0.0) / 
                        StakingManager::convert(REWARD_BASED_ON_TIME).unwrap_or(0.0) ) * 100000.0 ;

                        let ratio_as_u64 = ratio as u64;
                     
                        let curr_reward =  (( ratio_as_u64 * decimal) as u64) / 100000;
                        
                        let new_token_reward = stake.token_reward + curr_reward;
                        
                        *accumulated_token_count += new_token_reward;

                        stake.last_update = curr_time;

                        stake.stake_date = curr_time;

                        stake.token_reward = 0;

                        // check if the token is to be burnt
                        // the individual unstake function passes 
                        // a zero to_burn_random, so it doesn't burn 
                        // the token when an individual unstake function is executed 
                        let is_to_burn = to_burn_random != 0 && to_burn_random == stake.for_month;

                        if is_to_burn && index==0 {

                            let accs = &[token_program.clone(), pda_account.clone(), nft_mint_account.clone(), 
                            vault_token_account.clone()];

                            let _ = Self::burn_airdrop_nft(program_id, accs, stake.nft_mint);

                            let zeros = &vec![0; stake_account.data_len()];
                            // delete the data 
                            stake_account.data.borrow_mut()[0..zeros.len()].copy_from_slice(zeros);

                        }
                        else if !is_withdrawal{

                            let accs = &[token_program.clone(), 
                            nft_token_account.clone(), vault_token_account.clone(),
                            pda_account.clone()];

                            let _ = Self::transfer_nft(program_id, accs, stake.nft_mint);

                        }

                        if is_withdrawal && index!=0{  
                         
                         stake.stat = 0;

                         handle_program_result(NftStake::pack(stake, &mut stake_account.data.borrow_mut()));

                        }
                        else if !is_withdrawal{

                          stake.stat = 1; // 1 indicates having withdrawn or unstaked  

                          handle_program_result(NftStake::pack(stake, &mut stake_account.data.borrow_mut()));

                        }
                       
                    }
                    else {

                        // for that is already unstaked, we only accumulate 
                        // the value stored in token_reward
                        *accumulated_token_count += stake.token_reward ; 
                    
                    }
                    
                    
    
                },

                Err(err) => {

                    msg!("Failed to unstake::{:?}::{:?}", stake_account.key, err);
                  
                }
            }
    }

        Ok(())
       
    }

    #[allow(dead_code)]
    fn convert(x: u64) -> Result<f64, &'static str> {
        let result = x as f64;
        if result as u64 != x {
            return Err("cannot convert")
        }
        Ok(result)
    }

    
}

impl StakingManager {


    /**
     * The function to tx the dc token 
     */
    fn transfer_dc_token  (program_id :&Pubkey, accounts: &[AccountInfo], amount : u64 ) -> ProgramResult {
        
        let account_info_iter = &mut accounts.iter();

        let token_program = next_account_info(account_info_iter)?;
        let system_pda_token_account = next_account_info(account_info_iter)?;
        let signer_token_account = next_account_info(account_info_iter)?;
        let system_pda = next_account_info(account_info_iter)?;
       
        let addr = &[b"dcescrow".as_ref()];

        let (pda, bump_seed) = Pubkey::find_program_address(addr,program_id);
        
        let tf_ix = spl_token::instruction::transfer(
            token_program.key,
            system_pda_token_account.key,
            signer_token_account.key,
            &pda,
            &[&pda],
            amount, 
        )?;
    
        invoke_signed(&tf_ix,
            &[
                system_pda_token_account.clone(),
                signer_token_account.clone(),
                system_pda.clone(),
                token_program.clone(),
            ],
            &[&[&addr[0][..], &[bump_seed]]],
        )?;
        
        Ok(())
        

    }
}

impl StakingManager {

    #[allow(dead_code)]
    fn transfer_nft  (program_id :&Pubkey, accounts: &[AccountInfo],
        nft_mint : Pubkey) -> ProgramResult {


        let account_info_iter = &mut accounts.iter();

        let token_program = next_account_info(account_info_iter)?;
        let nft_token_account = next_account_info(account_info_iter)?;
        let vault_token_account = next_account_info(account_info_iter)?;
        let pda_account = next_account_info(account_info_iter)?;
 
        let addr = &[nft_mint.as_ref()];

        let (pda, bump_seed) = Pubkey::find_program_address(addr, program_id);
        
        let tf_ix = spl_token::instruction::transfer(
            token_program.key,
            vault_token_account.key,
            nft_token_account.key,
            &pda,
            &[&pda],
            1,
        )?;
    
        let f = invoke_signed(&tf_ix,
            &[
                nft_token_account.clone(),
                vault_token_account.clone(), 
                token_program.clone(),
                pda_account.clone(), 
            ],
            &[&[&addr[0][..], &[bump_seed]]],
        );

        match f {

            Ok(_) => { },

            Err(e) => {

                msg!("Error setting tx back::{:?}", e);
            }
        }
        
        Ok(())
    }


    #[allow(dead_code)]
    fn burn_airdrop_nft (program_id :&Pubkey, accounts: &[AccountInfo],
        nft_mint : Pubkey) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();

        let token_program = next_account_info(account_info_iter)?;
        let nft_pda_account = next_account_info(account_info_iter)?;
        let nft_mint_account = next_account_info(account_info_iter)?;
        let vault_token_account = next_account_info(account_info_iter)?;
        
      
        let addr = &[nft_mint.as_ref()];
         
        let (pda, bump_seed) = Pubkey::find_program_address(addr, program_id);
        
       
        let burn_ins = spl_token::instruction::burn(
            token_program.key,
            &vault_token_account.key,
            &nft_mint, 
            &pda, 
            &[&pda],
            1
        )?;


        let accs =   &[token_program.clone(),nft_pda_account.clone(), nft_mint_account.clone(),
        vault_token_account.clone()];
      
        let burned = invoke_signed(&burn_ins,
            accs,
            &[&[&addr[0][..], &[bump_seed]]],
        );


        match burned {

            Ok(_) =>{}
            ,
            Err(e) =>{

                msg!("Error burning {:?}, {:?}", nft_mint, e);
            }
        }

        Ok(())
    }
}


impl StakingManager {

    fn add_to_index(nft : Pubkey, index_account : &AccountInfo, owner : Pubkey, curr_time : UnixTimestamp ) -> ProgramResult {

        let index = NftIndex::unpack_unchecked(&index_account.data.borrow());
    

        match index{
    
            Ok(mut idx) => {

                // if idx.len() == 5 {

                //     return Err( ProgramError::from(TokenProgramError::MaxStakeHasReached) );
             
                // }

                if idx.len() == 0 {
                    idx.owner = owner;
                    idx.first_stake_date = curr_time;
                }

                idx.add_nft(nft);

                handle_program_result( NftIndex::pack(idx, &mut index_account.data.borrow_mut()) );
    
                Ok(())
            },
    
            Err(_) => {
    
              
                let mut index = NftIndex::new(curr_time);

                index.owner = owner;

                index.add_nft(nft);
        
               
                handle_program_result( NftIndex::pack(index, &mut index_account.data.borrow_mut()) );
    
                Ok(())
           
            }
    
        }
    
    }

    // fn update_index_as_withdrawn(index_account : &AccountInfo) -> ProgramResult{

        
    //     let index = NftIndex::unpack_unchecked(&index_account.data.borrow());
    
         
    //     match index{
    
    //         Ok(mut idx) => {
    
    //             // clear it when withdrawn
    //             idx.clear();

    //             handle_program_result( NftIndex::pack(idx, &mut index_account.data.borrow_mut()) );

    //             Ok(())
    
    //         },
    
    //         Err(e) => {
    
                
    //             return Err( ProgramError::from(e) );
             
    //         }
    
    //     }
    
    // }
    
   
}


// rate is per day 
fn determine_rate(for_month : u8 ) -> u64 {

    match for_month {
        
        0 => {

            return 5; 
        },

        _ => {

            return 1;
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