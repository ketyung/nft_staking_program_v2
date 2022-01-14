use solana_program::{
    pubkey::{Pubkey, PUBKEY_BYTES},
    program_pack::{Pack,Sealed},   
    program_error::ProgramError,
    clock::{UnixTimestamp},
};

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use std::convert::{TryFrom};


pub const NFT_SIZE_LIMIT : usize = 12;

#[derive(Clone, Debug, PartialEq)]
pub struct NftIndex {

    pub owner : Pubkey, 

    pub first_stake_date : UnixTimestamp,

    pub stat : u8, 

    nfts : Vec<Pubkey>,

}



impl NftIndex {

    pub fn new(date : UnixTimestamp) -> Self {

        NftIndex{

            owner : Pubkey::default(), 

            first_stake_date : date, 

            stat : 0, 
    
            nfts : Vec::with_capacity(NFT_SIZE_LIMIT),   
            
        }
    }
}


impl Sealed for NftIndex{}

impl Pack for NftIndex {

    const LEN: usize = PUBKEY_BYTES + 8 + 1 +  1 +  (PUBKEY_BYTES * NFT_SIZE_LIMIT) ;

    fn pack_into_slice(&self, dst: &mut [u8]) {

        const L : usize =  PUBKEY_BYTES + 8 + 1 + 1 + (PUBKEY_BYTES *  NFT_SIZE_LIMIT); 

        let output = array_mut_ref![dst, 0, L];

        let (owner, first_stake_date, stat, len, data_flat) = 
        mut_array_refs![output, PUBKEY_BYTES, 8, 1, 1,  (PUBKEY_BYTES * NFT_SIZE_LIMIT) ];

        *len = u8::try_from(self.nfts.len()).unwrap().to_le_bytes();       
        *first_stake_date = self.first_stake_date.to_le_bytes();
        *stat = u8::try_from(self.stat).unwrap().to_le_bytes();       
         
        owner.copy_from_slice(self.owner.as_ref());
      
        let mut offset = 0;

        for a in &self.nfts {

            let nft_flat = array_mut_ref![data_flat, offset, PUBKEY_BYTES];

            let (pack_nft, _) = mut_array_refs![nft_flat, PUBKEY_BYTES, 0];

            pack_nft.copy_from_slice(a.as_ref());

            offset += PUBKEY_BYTES;
        }

    }


    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {

        const L : usize = PUBKEY_BYTES + 8 + 1 + 1 + (PUBKEY_BYTES * NFT_SIZE_LIMIT) ; 

        let input = array_ref![src, 0, L];
        
        let (user, first_stake_date, stat, len, nfts) = array_refs![input, PUBKEY_BYTES ,8, 1, 1,  
        (PUBKEY_BYTES * NFT_SIZE_LIMIT) ];

        let len = u8::from_le_bytes(*len);
        let first_stake_date = i64::from_le_bytes(*first_stake_date);
        let stat = u8::from_le_bytes(*stat);

        let mut offset = 0 ;

        let mut new_nfts =  Vec::with_capacity(len as usize);

        for _ in 0..len {

            let pk = array_ref![nfts, offset, PUBKEY_BYTES];

            new_nfts.push(Pubkey::new_from_array(*pk));

            offset += PUBKEY_BYTES;
        }

        Ok(Self{
            owner : Pubkey::new_from_array(*user) ,
            first_stake_date : first_stake_date, 
            stat : stat, 
            nfts : new_nfts,
        })
    }

}


impl NftIndex {


    pub fn add_nft (&mut self,  pubkey : Pubkey){

        if self.nfts.len() < NFT_SIZE_LIMIT  {

            if !self.nfts.contains(&pubkey){

                self.nfts.push(pubkey);

            }
        }

    }


    pub fn remove_nft(&mut self, pubkey : Pubkey) {

        let idx = self.nfts.iter().position(|&r| r == pubkey);
        if idx.is_some() {

            self.nfts.remove(idx.unwrap());
        }
    }

  
    pub fn all(&self) -> Vec<Pubkey>{

        self.nfts.clone()
    }

    pub fn len(&self) -> usize{

        self.nfts.len()
    }


    pub fn clear(&mut self){

        self.nfts.clear();
    }

}
