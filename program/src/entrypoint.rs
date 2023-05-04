use spl_token::error::TokenError;
use {
    solana_program::{
        account_info::AccountInfo, entrypoint, entrypoint::ProgramResult,
        program_error::PrintProgramError, pubkey::Pubkey,
    },
};
use crate::processor::Processor;

entrypoint!(process_instruction);

fn process_instruction<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &'a [u8],
) -> ProgramResult {

    let mut processor = Processor {token_mints : Default::default()};
    if let Err(error) = processor.process_instruction(program_id, accounts, instruction_data) {
        error.print::<TokenError>();
        return Err(error);
    }
    Ok(())
}