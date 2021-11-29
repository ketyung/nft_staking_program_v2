use {

    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        pubkey::Pubkey,
        program_pack::{Pack},
        program_error::ProgramError,
        program::{invoke},
    },
    crate::vault::{TokenVaultStore, TokenVault},
    crate::staking_manager::{handle_program_result},

    
};

use crate::{error::TokenProgramError};

pub struct VaultManager;

impl VaultManager {

    pub fn add_vault (_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult{


        let account_info_iter = &mut accounts.iter();

        let signer_account = next_account_info(account_info_iter)?; 
        let token_mint = next_account_info(account_info_iter)?;
        let token_account = next_account_info(account_info_iter)?;
        let vault_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;
        

        let stored_vaults = TokenVaultStore::unpack_unchecked(&vault_account.data.borrow());
        
        match stored_vaults {

            Ok(mut s) => {
    
                let v = TokenVault{ mint: *token_mint.key, account : *token_account.key};

                if s.has(v.clone()) {

                    return Err( ProgramError::from(TokenProgramError::VaultAlreadyExists) );
             
                }

                s.add(v);


                msg!("current.size of vault store::{}", s.len());

                handle_program_result (TokenVaultStore::pack(s, &mut vault_account.data.borrow_mut()) );
    
            },

            Err(_) => {

                let mut s = TokenVaultStore::new();
            
                let v = TokenVault{ mint: *token_mint.key, account : *token_account.key};

                s.add(v);

                msg!("@err.x.current.size of vault store::{}", s.len());


                handle_program_result (TokenVaultStore::pack(s, &mut vault_account.data.borrow_mut()) );
    

            },
        }


        let addr = &[token_mint.key.as_ref()];
        let (pda, _bump_seed) = Pubkey::find_program_address(addr, _program_id);
       
        // change authority of the vault account to PDA 
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

        Ok(())

    }
}
