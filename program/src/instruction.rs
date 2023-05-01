use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;

pub enum ExchangeInstruction {
    SolToToken { amount: u64 },
    TokenToSol { amount: u64 },
    TokenToToken { amount: u64 }
}

#[derive(BorshDeserialize)]
pub struct InstructionData {
    amount: u64
}

impl ExchangeInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&variant, rest) = input.split_first().ok_or(ProgramError::InvalidInstructionData)?;
        let payload = InstructionData::try_from_slice(rest)?;
        match variant {
            0 => Ok( Self::SolToToken { amount: payload.amount } ),
            1 => Ok( Self::TokenToSol { amount: payload.amount } ),
            2 => Ok( Self::TokenToToken { amount: payload.amount }),
            _ => Err(ProgramError::InvalidInstructionData)
        }
    }
}



