#![cfg(all(target_arch = "bpf", not(feature = "no-entrypoint")))]

use {
    crate::{processor},
    solana_program::{
        account_info::AccountInfo, entrypoint, entrypoint::ProgramResult,
        pubkey::Pubkey,
    },
};


entrypoint!(process_instruction);
fn process_instruction<'a>(program_id: &'a Pubkey,accounts: &'a [AccountInfo<'a>],_instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) = processor::process_instruction(program_id, accounts, _instruction_data) {
       
        return Err(error);
    }
    Ok(())
}