use solana_program::{
    pubkey::{Pubkey, PUBKEY_BYTES},
    program_error::ProgramError,
    program_pack::{Pack,Sealed},
    clock::{Clock, UnixTimestamp},
    sysvar::Sysvar,  
   
    //msg, 
};
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

pub const DC_TOKEN_DECIMAL : u32 = 6;

#[derive(Clone, Debug, PartialEq)]
pub struct Sys {

    pub token_account : Pubkey, 
    
    pub token_pda : Pubkey,

    pub token_mint : Pubkey, 

    pub date_added : UnixTimestamp, 

}

impl Sys {

    pub fn new() -> Self {

        Sys {

            token_account : Pubkey::default(), 

            token_pda : Pubkey::default(), 

            token_mint : Pubkey:: default(), 

            date_added : Clock::get().unwrap().unix_timestamp,   
        }
    
    }
}


impl Sealed for Sys{}


const SYS_DATA_SIZE : usize = PUBKEY_BYTES + PUBKEY_BYTES + PUBKEY_BYTES + 8;

impl Pack for Sys {

    const LEN: usize = SYS_DATA_SIZE;


    fn pack_into_slice(&self, dst: &mut [u8]) {

        let output = array_mut_ref![dst, 0, SYS_DATA_SIZE];
       
        let (token_account, token_pda, token_mint, date_added) = 
        mut_array_refs![ output,PUBKEY_BYTES,PUBKEY_BYTES,PUBKEY_BYTES,8];

        token_account.copy_from_slice(self.token_account.as_ref());
        token_pda.copy_from_slice(self.token_pda.as_ref());
        token_mint.copy_from_slice(self.token_mint.as_ref());
        *date_added = self.date_added.to_le_bytes();
     

    }


    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
   
        let input = array_ref![src, 0, SYS_DATA_SIZE];
       
        let (token_account, token_pda, token_mint,  date_added) = 
       
        array_refs![input, PUBKEY_BYTES,PUBKEY_BYTES,PUBKEY_BYTES, 8];

        let token_account = Pubkey::new_from_array(*token_account);
        let token_pda = Pubkey::new_from_array(*token_pda);
        let token_mint = Pubkey::new_from_array(*token_mint);
        
        
        let date_added = i64::from_le_bytes(*date_added);
       
        Ok( Sys{
           
            token_account : token_account,

            token_pda : token_pda,

            token_mint : token_mint,
            
            date_added : date_added, 
        })

    }

}







#[derive(Clone, Debug, PartialEq)]
pub struct NftStake {

    pub nft_mint : Pubkey,

    pub owner : Pubkey,

    pub for_month : u8, 

    pub pda : Pubkey,

    pub nft_token_account :  Pubkey, 

    pub rate : u64, 
    
    pub token_reward : u64, 

    pub stake_date : UnixTimestamp, 

    pub last_update : UnixTimestamp, 

    pub stat : u8,

    pub meta_key : Pubkey,

    pub created : UnixTimestamp,

    pub vault_account : Pubkey,


}

impl NftStake {

    pub fn new(date : UnixTimestamp) -> Self {

        NftStake {
            nft_mint : Pubkey::default(),
            owner : Pubkey::default(),
            for_month : 0,
            pda : Pubkey::default(),
            nft_token_account : Pubkey::default(), 
            rate : 0,
            token_reward : 0,
            stake_date : date, 
            last_update : date, 
            stat : 0, 
            created : date, 
            meta_key : Pubkey::default(),
            vault_account : Pubkey::default(),
            
        }
    }

}


impl Sealed for NftStake{}

const NFTSTAKE_DATA_SIZE : usize = 
PUBKEY_BYTES + PUBKEY_BYTES + 1 + PUBKEY_BYTES + PUBKEY_BYTES + 8 + 8 + 8 + 8 + 1 + 8 + PUBKEY_BYTES + PUBKEY_BYTES;


impl Pack for NftStake {


    const LEN: usize = NFTSTAKE_DATA_SIZE;


    fn pack_into_slice(&self, dst: &mut [u8]) {

        let output = array_mut_ref![dst, 0, NFTSTAKE_DATA_SIZE];
       
        let (nft_mint, owner, for_month, pda, 
            nft_token_account, 
            rate, token_reward, stake_date, last_update, 
            stat, created, meta_key, vault_account) = 
        mut_array_refs![ output,PUBKEY_BYTES,PUBKEY_BYTES,1,PUBKEY_BYTES,PUBKEY_BYTES,8,8, 8, 8, 1, 
        8, PUBKEY_BYTES, PUBKEY_BYTES];


        nft_mint.copy_from_slice(self.nft_mint.as_ref());
        owner.copy_from_slice(self.owner.as_ref());
        *for_month = self.for_month.to_le_bytes();
        pda.copy_from_slice(self.pda.as_ref());
        nft_token_account.copy_from_slice(self.nft_token_account.as_ref());
        

        *rate = self.rate.to_le_bytes();

        *token_reward = self.token_reward.to_le_bytes();

        *stake_date = self.stake_date.to_le_bytes();
        *last_update = self.last_update.to_le_bytes();
        *stat = self.stat.to_le_bytes();

        *created = self.created.to_le_bytes();

        meta_key.copy_from_slice( self.meta_key.as_ref());
        vault_account.copy_from_slice( self.vault_account.as_ref());
        
    }


    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
   
        let input = array_ref![src, 0, NFTSTAKE_DATA_SIZE];
       
        let (nft_mint,owner, for_month, pda, nft_token_account, 
            rate, token_reward, stake_date, last_update, stat, 
            created, meta_key, vault_account) = 
       
        array_refs![input, PUBKEY_BYTES,PUBKEY_BYTES, 1, PUBKEY_BYTES,PUBKEY_BYTES, 
        8, 8, 8, 8 , 1, 8, PUBKEY_BYTES, PUBKEY_BYTES];

        let nft_mint = Pubkey::new_from_array(*nft_mint);
        let owner = Pubkey::new_from_array(*owner);
        let for_month = u8::from_le_bytes(*for_month);
        let pda = Pubkey::new_from_array(*pda);
        let nft_token_account = Pubkey::new_from_array(*nft_token_account);
        
        let rate = u64::from_le_bytes(*rate);
        let token_reward = u64::from_le_bytes(*token_reward);
        let stake_date = i64::from_le_bytes(*stake_date);
        let last_update = i64::from_le_bytes(*last_update);
        let stat =  u8::from_le_bytes(*stat);
        let created = i64::from_le_bytes(*created);
        let meta_key = Pubkey::new_from_array(*meta_key);
        let vault_account = Pubkey::new_from_array(*vault_account);
        

        Ok( NftStake{
            nft_mint : nft_mint,
            owner : owner,
            for_month : for_month,
            pda : pda,
            nft_token_account : nft_token_account, 
            rate : rate,
            token_reward : token_reward, 
            stake_date : stake_date, 
            last_update : last_update, 
            stat : stat, 
            created : created,
            meta_key : meta_key,
            vault_account : vault_account, 
            
        })
    }
}