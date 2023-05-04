use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;

pub enum Instruction {
    Deposit {
        amount: u64,
        pda_seed: [u8; 16],
        bump_seed: u8
    },
    Withdraw { amount: u64,
        pda_seed: [u8; 16],
        bump_seed: u8
    },
    ExchangeSolToToken { amount: u64 },
    ExchangeTokenToSol { amount: u64 },
    ExchangeTokenToToken { amount: u64 }
}

#[derive(BorshDeserialize)]
pub struct InstructionData {
    amount: u64,
    pda_seed: String,
    bump_seed: u8
}

impl Instruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&variant, rest) = input.split_first().ok_or(ProgramError::InvalidInstructionData)?;
        let payload = InstructionData::try_from_slice(rest)?;
        match variant {
            0 => Ok( Self::Deposit {
                amount: payload.amount,
                pda_seed: <[u8; 16]>::try_from(payload.pda_seed.into_bytes()).unwrap(),
                bump_seed: payload.bump_seed
            }),
            1 => Ok( Self::Withdraw {
                amount: payload.amount,
                pda_seed: <[u8; 16]>::try_from(payload.pda_seed.into_bytes()).unwrap(),
                bump_seed: payload.bump_seed
            }),
            2 => Ok( Self::ExchangeSolToToken { amount: payload.amount } ),
            3 => Ok( Self::ExchangeTokenToSol { amount: payload.amount } ),
            4 => Ok( Self::ExchangeTokenToToken { amount: payload.amount }),
            _ => Err(ProgramError::InvalidInstructionData)
        }
    }
}



