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


pub struct StakingManager {}

impl StakingManager {

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


       // msg!("meta.acc::.key::{:?}", meta_account.key);

        if !signer_account.is_signer {

            return Err(ProgramError::MissingRequiredSignature);
        }


        if stake_account.owner != program_id {

            return Err( ProgramError::IncorrectProgramId );
        }

        if index_account.owner != program_id {

            return Err( ProgramError::IncorrectProgramId );       
        }

        //Self::verify_nft_authority(&nft_mint)?;


        // create PDA based on the NFT mint
        let addr = &[nft_mint.key.as_ref()];
        
        let (pda, _bump_seed) = Pubkey::find_program_address(addr, program_id);
       
        
        let stored_stake = NftStake::unpack_unchecked(&stake_account.data.borrow());
        
        let curr_time =  Clock::get().unwrap().unix_timestamp;

        match stored_stake {

            Ok(_) => {
    
                //assert_eq!(stake.owner, *signer_account.key);

                msg!("Ok.created.new:");

                let mut stake = NftStake::new(curr_time);
                stake.for_month = for_month;
                stake.nft_mint = *nft_mint.key;
                stake.nft_token_account = *token_account.key;
                
                stake.rate = determine_rate(for_month);
                stake.pda = pda;
                stake.owner = *signer_account.key; 
                stake.meta_key = *meta_account.key; 
              
               
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
              
                handle_program_result( NftStake::pack(stake, &mut stake_account.data.borrow_mut()) );
               
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
        token_decimal : u32, count : u8, random_number : u8 )  -> ProgramResult{

        let account_info_iter = &mut accounts.iter();

        let signer_account = next_account_info(account_info_iter)?;
        let dc_token_mint = next_account_info(account_info_iter)?;
        let dc_token_account = next_account_info(account_info_iter)?;
        let dc_pda_account = next_account_info(account_info_iter)?;
        let signer_token_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?; 
        let index_account = next_account_info(account_info_iter)?;

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

        if Self::already_withdrawn(&index_account){

            return Err(ProgramError::from(TokenProgramError::AlreadyWithdrawn));
        }


        

        let mut accumulated_token_count : u64 = 0; 


        // instructions for unstaking all the 7 staked NFTs
        // the instruction will burn the monthly airdropped NFT during unstaking
        
        for _n in 0..count {
            
           
            let stake_account = next_account_info(account_info_iter)?;

            if stake_account.owner != program_id{

                return Err(ProgramError::IncorrectProgramId);
            }


            let nft_token_account = next_account_info(account_info_iter)?;
            let vault_token_account = next_account_info(account_info_iter)?;
            let nft_pda_account = next_account_info(account_info_iter)?;
            let nft_mint_account = next_account_info(account_info_iter)?;
            

            /*
            let token_program = next_account_info(account_info_iter)?;
            let stake_account = next_account_info(account_info_iter)?;
            let nft_token_account = next_account_info(account_info_iter)?;
            let vault_token_account = next_account_info(account_info_iter)?;
            let pda_account = next_account_info(account_info_iter)?;
            */

            
            let accs = &[token_program.clone(),  
            stake_account.clone(), nft_token_account.clone(),
            vault_token_account.clone(), nft_pda_account.clone(), 
            nft_mint_account.clone()];

         
            handle_program_result( Self::unstake_account(&program_id, accs, &mut accumulated_token_count, 
                token_decimal, random_number, true) );

          //  msg!("acc.count::{}", accumulated_token_count);


        }


        //assert_eq!(count, 100);
       
        //msg!("accumulated.tk.cnt::{}", accumulated_token_count);

        // uncomment this for testing so can view each log message at console
        // assert_eq!(accumulated_token_count, 100);

        /*
        let token_program = next_account_info(account_info_iter)?;
        let system_pda_token_account = next_account_info(account_info_iter)?;
        let signer_token_account = next_account_info(account_info_iter)?;
        let system_pda = next_account_info(account_info_iter)?;
        */
       
        // accumulated_token_count already includes 
        // the token decimal, so there is NO need to multiply 
        // token decimal
        handle_program_result( Self::transfer_dc_token(&program_id, 
            &[token_program.clone(),  dc_token_account.clone(), 
            signer_token_account.clone(), dc_pda_account.clone()],
            accumulated_token_count ) );
        
    
        Self::update_index_as_withdrawn(&index_account)?;

        Ok(())
    }
}




const REWARD_BASED_ON_TIME : u64 = 86400; 

impl StakingManager{

     pub fn unstake_account (program_id: &Pubkey, accounts: &[AccountInfo],
        accumulated_token_count : &mut u64, token_decimal : u32, to_burn_random : u8, 
        is_withdrawal : bool) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();

        let token_program = next_account_info(account_info_iter)?;
        let stake_account = next_account_info(account_info_iter)?;
        let nft_token_account = next_account_info(account_info_iter)?;
        let vault_token_account = next_account_info(account_info_iter)?;
        let pda_account = next_account_info(account_info_iter)?;
        let nft_mint_account = next_account_info(account_info_iter)?;
       // let signer_account = next_account_info(account_info_iter)?;

       
        //msg!("Unstaking....stake_account::{:?}", stake_account.key);
        
        // unstake only if the owner is program_id
        if stake_account.owner == program_id {

            let stored_stake = NftStake::unpack_unchecked(&stake_account.data.borrow());
   
            match stored_stake {

                Ok(mut stake) => {
        

                    let curr_time = Clock::get().unwrap().unix_timestamp;

                    let rate = determine_rate(stake.for_month);

                    
                    // if it's staked 
                    if  stake.stat == 0 {

                        // time difference calculated 
                        // from the stake_date till current datetime
                        let time_diff = curr_time - stake.stake_date;
                    
                        let ten : u64 = 10;
    
                        let decimal = ten.pow(token_decimal);
        
                        let time_diff_earned : u64 = u64::try_from(time_diff).unwrap() * rate;
    
                       // let ratio = StakingManager::convert( time_diff_earned).unwrap_or(0.0) / 
                       // StakingManager::convert(REWARD_BASED_ON_TIME).unwrap_or(0.0);

                        let ratio = (StakingManager::convert( time_diff_earned).unwrap_or(0.0) / 
                        StakingManager::convert(REWARD_BASED_ON_TIME).unwrap_or(0.0) ) * 100000.0 ;

                        let ratio_as_u64 = ratio as u64;
                     
                        let curr_reward =  (( ratio_as_u64 * decimal) as u64) / 100000;

                        
                        let new_token_reward = stake.token_reward + curr_reward;

                        stake.token_reward = new_token_reward; 

                        
                        *accumulated_token_count += new_token_reward;

                        stake.last_update = curr_time;

                        stake.stat = 1; // 1 indicates having withdrawn or unstaked

                        // check if the token is to be burnt
                        // the individual unstake function passes 
                        // a zero to_burn_random, so it doesn't burn 
                        // the token when an individual unstake function is executed 
                        let is_to_burn = to_burn_random != 0 && to_burn_random == stake.for_month;

                        if is_to_burn {

                            let accs = &[token_program.clone(), pda_account.clone(), nft_mint_account.clone(), 
                            vault_token_account.clone()];

                            let _ = Self::burn_airdrop_nft(program_id, accs, stake.nft_mint);

                        }
                        else {

                            let accs = &[token_program.clone(), 
                            nft_token_account.clone(), vault_token_account.clone(),
                            pda_account.clone()];

                            let _ = Self::transfer_nft(program_id, accs, stake.nft_mint);

                        }
                    

                        if is_withdrawal {

                            let zeros = &vec![0; stake_account.data_len()];
                            // delete the data 
                            stake_account.data.borrow_mut()[0..zeros.len()].copy_from_slice(zeros);
                    
                        }
                        else {

                            handle_program_result(NftStake::pack(stake, &mut stake_account.data.borrow_mut()));

                        }
                       
                    }
                    else {

                        // for that is already unstaked, we only accumulate 
                        // the value stored in token_reward
                        *accumulated_token_count += stake.token_reward ; 
                    
                      //  msg!("stake.token_reward::{}", stake.token_reward);
                    
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


    fn transfer_dc_token  (program_id :&Pubkey, accounts: &[AccountInfo], amount : u64 ) -> ProgramResult {
        
        let account_info_iter = &mut accounts.iter();

        let token_program = next_account_info(account_info_iter)?;
        let system_pda_token_account = next_account_info(account_info_iter)?;
        let signer_token_account = next_account_info(account_info_iter)?;
        let system_pda = next_account_info(account_info_iter)?;
       
       // msg!("tx.dc.token to :{:?} amount :{}", signer_token_account.key, amount);

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

   // #[deprecated]
    #[allow(dead_code)]
    fn transfer_nft  (program_id :&Pubkey, accounts: &[AccountInfo],
        nft_mint : Pubkey) -> ProgramResult {


        let account_info_iter = &mut accounts.iter();

        /* // follow this order 
        let accs = &[token_program.clone(), 
        nft_token_account.clone(), vault_token_account.clone(), 
        pda_account.clone() ];

        */

        let token_program = next_account_info(account_info_iter)?;
        let nft_token_account = next_account_info(account_info_iter)?;
        let vault_token_account = next_account_info(account_info_iter)?;
        let pda_account = next_account_info(account_info_iter)?;

        /*
        msg!("rs.token_prog::{:?}", token_program.key);
        msg!("rs.nft_token_acc::{:?}", nft_token_account.key);
        msg!("rs.vault_token_acc::{:?}", vault_token_account.key);
        msg!("rs.pda_acc::{:?}", pda_account.key);
        msg!("rs.pda::{}", pda);
        */

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

            Ok(_) => {

               // msg!("tx back from::{:?} to :{:?}", vault_token_account.key,  nft_token_account.key);
          
            },

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
        
        /*
        msg!("burn.nft  : nft:{:?}", nft_mint);
        msg!("burn.nft : token_prg:{:?}", token_program.key);
        msg!("burn.nft : nft_mint_acc:{:?}", nft_mint_account.key);
        msg!("burn.nft : nft_pda_acc:{:?}", nft_pda_account.key);
         */
        
        // create PDA based on the NFT mint
        let addr = &[nft_mint.as_ref()];
         
        let (pda, bump_seed) = Pubkey::find_program_address(addr, program_id);
        
       // msg!("pda:{:?}", pda);
       
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
      
        //let burned = invoke(&burn_ins, accs);

        let burned = invoke_signed(&burn_ins,
            accs,
            &[&[&addr[0][..], &[bump_seed]]],
        );


        match burned {

            Ok(_) =>{

               // msg!("Burned :{:?}", s);
            }
            ,
            Err(e) =>{

                msg!("Error burning {:?}, {:?}", nft_mint, e);
            }
        }

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

    fn update_index_as_withdrawn(index_account : &AccountInfo) -> ProgramResult{

        
        let index = NftIndex::unpack_unchecked(&index_account.data.borrow());
    
         
        match index{
    
            Ok(mut idx) => {
    
                idx.stat = 1 ;

                handle_program_result( NftIndex::pack(idx, &mut index_account.data.borrow_mut()) );

                Ok(())
    
            },
    
            Err(e) => {
    
                
                return Err( ProgramError::from(e) );
             
            }
    
        }
    
    }
    
    fn already_withdrawn(index_account : &AccountInfo) -> bool {

        let index = NftIndex::unpack_unchecked(&index_account.data.borrow()).unwrap();
        
        return index.stat == 1;

    }
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