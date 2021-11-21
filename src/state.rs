use solana_program::{
    pubkey::{Pubkey, PUBKEY_BYTES},
    program_error::ProgramError,
    program_pack::{Pack,Sealed},
    clock::{UnixTimestamp},
    //msg, 
};
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

#[derive(Clone, Debug, PartialEq)]
pub struct NftStake {

    pub nft_mint : Pubkey,

    pub owner : Pubkey,

    pub for_month : u8, 

    pub pda : Pubkey,

    pub rate : u64, 
    
    pub token_reward : u64, 

    pub stake_date : UnixTimestamp, 

    pub last_update : UnixTimestamp, 

    pub stat : u8,


}

impl NftStake {

    pub fn new(date : UnixTimestamp) -> Self {

        NftStake {
            nft_mint : Pubkey::default(),
            owner : Pubkey::default(),
            for_month : 0,
            pda : Pubkey::default(),
            rate : 0,
            token_reward : 0,
            stake_date : date, 
            last_update : date, 
            stat : 0, 
            
        }
    }
}


impl Sealed for NftStake{}

const NFTSTAKE_DATA_SIZE : usize = 
PUBKEY_BYTES + PUBKEY_BYTES + 1 + PUBKEY_BYTES + 8 + 8 + 8 + 8 + 1;


impl Pack for NftStake {


    const LEN: usize = NFTSTAKE_DATA_SIZE;


    fn pack_into_slice(&self, dst: &mut [u8]) {

        let output = array_mut_ref![dst, 0, NFTSTAKE_DATA_SIZE];
       
        let (nft_mint, owner, for_month, pda, rate, token_reward, stake_date, last_update, stat) = 
        mut_array_refs![ output,PUBKEY_BYTES,PUBKEY_BYTES,1,PUBKEY_BYTES,8,8, 8, 8, 1];


        nft_mint.copy_from_slice(self.nft_mint.as_ref());
        owner.copy_from_slice(self.owner.as_ref());
        *for_month = self.for_month.to_le_bytes();
        pda.copy_from_slice(self.pda.as_ref());

        *rate = self.rate.to_le_bytes();

        *token_reward = self.token_reward.to_le_bytes();

        *stake_date = self.stake_date.to_le_bytes();
        *last_update = self.last_update.to_le_bytes();
        *stat = self.stat.to_le_bytes();


    }


    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
   
        let input = array_ref![src, 0, NFTSTAKE_DATA_SIZE];
       
        let (nft_mint,owner, for_month, pda, rate, token_reward, stake_date, last_update, stat) = 
       
        array_refs![input, PUBKEY_BYTES,PUBKEY_BYTES, 1, PUBKEY_BYTES, 8, 8, 8, 8 , 1];

        let nft_mint = Pubkey::new_from_array(*nft_mint);
        let owner = Pubkey::new_from_array(*owner);
        let for_month = u8::from_le_bytes(*for_month);
        let pda = Pubkey::new_from_array(*pda);
        let rate = u64::from_le_bytes(*rate);
        let token_reward = u64::from_le_bytes(*token_reward);
        let stake_date = i64::from_le_bytes(*stake_date);
        let last_update = i64::from_le_bytes(*last_update);
        let stat =  u8::from_le_bytes(*stat);

        Ok( NftStake{
            nft_mint : nft_mint,
            owner : owner,
            for_month : for_month,
            pda : pda,
            rate : rate,
            token_reward : token_reward, 
            stake_date : stake_date, 
            last_update : last_update, 
            stat : stat, 
            
        })
    }
}