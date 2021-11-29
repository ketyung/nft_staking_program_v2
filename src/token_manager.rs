use {

    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        pubkey::Pubkey,
        program_pack::{Pack},
        program_error::ProgramError,
        program::{invoke_signed, invoke},
    },
    crate::state::{Sys},
    crate::staking_manager::{handle_program_result},
    
};

pub struct TokenManager {}

impl TokenManager {


    pub fn mint_token (_program_id: &Pubkey, accounts: &[AccountInfo],
        amount : u64, decimal : u64, _finalized : bool) -> ProgramResult{


        let account_info_iter = &mut accounts.iter();

        let signer_account = next_account_info(account_info_iter)?;
        
        let token_mint = next_account_info(account_info_iter)?; 
    
        let token_account = next_account_info(account_info_iter)?; 
        
        let token_program = next_account_info(account_info_iter)?;

        msg!("Minting amount :{}, finalized : {}", amount, _finalized);
           
        msg!("signer.key::{:?}", signer_account.key);
        msg!("token.mint::{:?}", token_mint.key);
        msg!("token.acc::{:?}", token_account.key);
        msg!("token.prog::{:?}", token_program.key);
        
    
        if !signer_account.is_signer {

            return Err(ProgramError::MissingRequiredSignature);
        }

    
        if *token_account.owner != spl_token::id() {
            msg!("token_account.owner is {:?}, whereas spl_token prog id is::{:?}", 
            token_account.owner, spl_token::id());
            return Err(ProgramError::IncorrectProgramId);
        }
    

        let mut token_count = amount * decimal;

        // decimal 0 is NFT
        if decimal == 0 {

            token_count = 1;
        }

        let ix = spl_token::instruction::mint_to(
            token_program.key,
            token_mint.key,
            token_account.key,
            signer_account.key,
            &[],
            token_count,
        )?;
    
    
        let signers = &[
            signer_account.key.as_ref(),
        ];
    
        msg!("Going.to.mint.token....");

        invoke_signed(
            &ix,
            &[
                token_mint.clone(),
                token_account.clone(),
                signer_account.clone(),
                token_program.clone(),
            ],
            &[signers],
        )?;

        if _finalized {

            let freeze_ix = spl_token::instruction::close_account(
                token_program.key, token_account.key, 
                token_mint.key, token_account.key, &[])?;

        
            invoke_signed(
                &freeze_ix,
                &[
                    token_mint.clone(),
                    token_account.clone(),
                    signer_account.clone(),
                    token_program.clone(),
                ],
                &[signers],
            )?;
    

        }

        msg!("token.minted!...");
        

        Ok(())


    }
}

impl TokenManager {

    pub fn create_sys (program_id: &Pubkey, accounts: &[AccountInfo] ) -> ProgramResult{

        let account_info_iter = &mut accounts.iter();

        let signer_account = next_account_info(account_info_iter)?;
        
        let sys_account = next_account_info(account_info_iter)?;
       
        let dc_token_account = next_account_info(account_info_iter)?;
      
        let dc_token_mint = next_account_info(account_info_iter)?;

        let pda_account = next_account_info(account_info_iter)?;
      
        let token_program = next_account_info(account_info_iter)?;
      
        if !signer_account.is_signer {

            return Err(ProgramError::MissingRequiredSignature);
        }

        
        if sys_account.owner != program_id {

            return Err( ProgramError::IncorrectProgramId );
        }

        if *dc_token_mint.owner != spl_token::id() {

           // msg!("incorrect.splToken.{:?},{:?}!!", dc_token_account.owner , spl_token::id());

            return Err( ProgramError::IncorrectProgramId );
        }


      //  msg!("sys_token_account: {:?}", dc_token_account.key);
      //  msg!("sys_token_mint: {:?}", dc_token_mint.key);
        
        let addr = &[b"dcescrow".as_ref()];

        let (pda, _bump_seed) = Pubkey::find_program_address(addr , program_id);

        assert_eq!(pda, *pda_account.key);

        let stored_sys = Sys::unpack_unchecked(&dc_token_account.data.borrow());
        
        match stored_sys {

            Ok(_) => {
    
                let mut sys = Sys::new();

                sys.token_account = *dc_token_account.key ;
                sys.token_pda = pda ;
                sys.token_mint = *dc_token_mint.key ;

                handle_program_result (Sys::pack(sys, &mut sys_account.data.borrow_mut()) );
    
            },
    
            Err(_) => {
    
   
                let mut sys = Sys::new();

                sys.token_account = *dc_token_account.key ;
                sys.token_pda = pda ;
                sys.token_mint = *dc_token_mint.key ;
                
                handle_program_result (Sys::pack(sys, &mut sys_account.data.borrow_mut()) );
   
            } 
            
        }


        // change authority of the vault account to PDA 
        let tf_to_pda_ix = spl_token::instruction::set_authority(
            token_program.key,
            dc_token_account.key,
            Some(&pda), 
            spl_token::instruction::AuthorityType::AccountOwner,
            signer_account.key,
            &[&signer_account.key],
        )?;
      
        
        invoke(
            &tf_to_pda_ix,
            &[
                dc_token_account.clone(),
                signer_account.clone(),
                token_program.clone(),
            ],
        )?;

    
        Ok(())

    }
}


impl TokenManager {


    pub fn transfer_token_to_sys(_program_id: &Pubkey, accounts: &[AccountInfo], 
        _amount : u64, _decimal : u32) -> ProgramResult{

        let account_info_iter = &mut accounts.iter();

        let signer_account = next_account_info(account_info_iter)?;
       
        let signer_token_account = next_account_info(account_info_iter)?; 
       
        let dc_token_account = next_account_info(account_info_iter)?;
      
        let token_program = next_account_info(account_info_iter)?;

        if !signer_account.is_signer {

            return Err(ProgramError::MissingRequiredSignature);
        }


        if *dc_token_account.owner != spl_token::id() {

             return Err( ProgramError::IncorrectProgramId );
         }
 

        let ten : u64 = 10;

        let decimal = ten.pow(_decimal);
    
        let token_count : u64 = _amount * decimal;

        msg!("token.account:: {:?}::{}", dc_token_account.key, token_count);

        
        let tf_ix = spl_token::instruction::transfer(
            token_program.key,
            signer_token_account.key,
            dc_token_account.key,
            &signer_account.key,
            &[&signer_account.key],
            token_count,
        )?;
    
        // the number of accounts involved must all 
        // passed in the 2nd param array
        invoke(
            &tf_ix,
            &[
                signer_token_account.clone(),
                signer_account.clone(),
                dc_token_account.clone(),
                token_program.clone(),
            ],
        )?;


        Ok(())

    }

}