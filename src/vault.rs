use solana_program::{
    pubkey::{Pubkey, PUBKEY_BYTES},
    program_error::ProgramError,
    program_pack::{Pack,Sealed},
};

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use std::convert::{TryFrom};


#[derive(Clone, Debug)]
pub struct TokenVault {

    pub mint :  Pubkey, 

    pub account : Pubkey, 

}


impl PartialEq for TokenVault {
    fn eq(&self, other: &Self) -> bool {
        self.mint == other.mint
    }
}

pub struct TokenVaultStore {

    vaults : Vec<TokenVault>,
}

pub const VAULT_SIZE_LIMIT : usize = 12;


impl TokenVaultStore {

    pub fn new() -> Self {

        TokenVaultStore{

            vaults : Vec::with_capacity(VAULT_SIZE_LIMIT),   
        }
    }
}


impl TokenVaultStore {

    pub fn add (&mut self, vault : TokenVault) {

        if self.vaults.len() < VAULT_SIZE_LIMIT {

            if !self.vaults.contains(&vault) {

                self.vaults.push(vault);
            }

        }

    }


    pub fn len(&self) -> usize {

        return self.vaults.len();
    }


    pub fn has (&self, vault : TokenVault) -> bool {

        return self.vaults.contains(&vault);

    }
}





impl Sealed for TokenVaultStore{}

impl Pack for TokenVaultStore {

    const LEN: usize = 1 + ((PUBKEY_BYTES + PUBKEY_BYTES) * VAULT_SIZE_LIMIT) ;

    fn pack_into_slice(&self, dst: &mut [u8]) {

        const L : usize =  1 + ((PUBKEY_BYTES + PUBKEY_BYTES) * VAULT_SIZE_LIMIT) ;


        let output = array_mut_ref![dst, 0, L];

        let (len, data_flat) = 
        mut_array_refs![output, 1,  ((PUBKEY_BYTES + PUBKEY_BYTES) * VAULT_SIZE_LIMIT) ];

        *len = u8::try_from(self.vaults.len()).unwrap().to_le_bytes();      

        let mut offset = 0;

        for a in &self.vaults {


            let vault_flat = array_mut_ref![data_flat, offset, PUBKEY_BYTES * 2];

            let (mint, account) = mut_array_refs![vault_flat, PUBKEY_BYTES,  PUBKEY_BYTES];

            mint.copy_from_slice(a.mint.as_ref());

            account.copy_from_slice(a.account.as_ref());

            offset += PUBKEY_BYTES * 2 ;
        }

    }


    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {

        const L : usize =  1 + ((PUBKEY_BYTES + PUBKEY_BYTES) * VAULT_SIZE_LIMIT) ;

        let input = array_ref![src, 0, L];
        
        let (len, vaults_flat) = array_refs![input, 1,  ((PUBKEY_BYTES + PUBKEY_BYTES) * VAULT_SIZE_LIMIT) ];

        let len = u8::from_le_bytes(*len);
        let mut offset = 0 ;

        let mut new_vaults =  Vec::with_capacity(len as usize);

        for _ in 0..len {

            
            let iv_flat = array_ref![vaults_flat, offset, PUBKEY_BYTES * 2];

            let (mint,account) = array_refs![iv_flat, PUBKEY_BYTES, PUBKEY_BYTES];

            new_vaults.push (

                TokenVault {

                    mint : Pubkey::new_from_array(*mint),
                    account : Pubkey::new_from_array(*account), 
                }
            );


            offset += PUBKEY_BYTES * 2;


        }

        Ok(Self{
            vaults : new_vaults,
        })
    }

}



