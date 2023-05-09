use std::collections::HashSet;
use solana_program::
{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    native_token::LAMPORTS_PER_SOL,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction
};
use spl_token::{
    self,
    instruction as token_instruction,
};
use chainlink_solana as chainlink;
use num_traits::{FromPrimitive, Pow, ToPrimitive};
use solana_program::program::invoke_signed;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use crate::error::TokenError;
use crate::instruction::Instruction;

pub(crate) struct Processor {
    pub(crate) token_mints: HashSet<String>  // token mint ID's
}

impl Processor {
    // TOKENS ARE ALWAYS TRANSFERRED :
    // FROM A TO A (FROM CLIENT TO EXCHANGE)
    // FROM B TO B (FROM EXCHANGE TO CLIENT)
    #[allow(non_snake_case)]
    pub fn process_instruction<'a>(
        &mut self,
        program_id: &'a Pubkey,
        accounts: &'a [AccountInfo<'a>],
        instruction_data: &'a [u8],
    ) -> ProgramResult {
        let instruction = Instruction::unpack(instruction_data)?;

        match instruction {
            Instruction::Deposit { amount, pda_seed, bump_seed } => {
                Self::deposit(self, program_id, accounts, amount, &pda_seed, bump_seed)?;
                Ok(())
            }

            Instruction::Withdraw { amount, pda_seed, bump_seed } => {
                Self::withdraw(self, program_id, accounts, amount, &pda_seed, bump_seed)?;
                Ok(())
            }

            Instruction::ExchangeSolToToken { amount } => {
                Self::exchange_sol_to_token(program_id, accounts, amount)?;
                Ok(())
            }

            Instruction::ExchangeTokenToSol { amount } => {
                Self::exchange_token_to_sol(program_id, accounts, amount)?;
                Ok(())
            }

            Instruction::ExchangeTokenToToken { amount } => {
                Self::exchange_token_to_token(program_id, accounts, amount)?;
                Ok(())
            }
        }
    }

    fn deposit(
        &mut self,
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
        pda_seed: &[u8;16],
        bump_seed: u8

    ) -> ProgramResult
    {
        let accounts_iter = &mut accounts.iter();

        let token_program = next_account_info(accounts_iter)?;
        let mint = next_account_info(accounts_iter)?;
        let exchange_wallet = next_account_info(accounts_iter)?;
        let exchange_token_account = next_account_info(accounts_iter)?;
        let pda_token_account = next_account_info(accounts_iter)?;
        let exchange_program_account = next_account_info(accounts_iter)?;

        if !exchange_wallet.is_signer {
            msg!("Missing required signature");
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mint_key = mint.key.to_string();
        let key = mint_key;
        if !self.token_mints.contains(key.as_str()) {

            self.token_mints.insert(key);

            let account_len: usize = 1000;
            let rent = Rent::get()?;
            let rent_lamports = rent.minimum_balance(account_len);

            invoke_signed(
                &system_instruction::create_account(
                    exchange_wallet.key,
                    pda_token_account.key,
                    rent_lamports,
                    account_len.to_u64().unwrap(),
                    exchange_program_account.key
                ),
                &[
                    exchange_wallet.clone(),
                    pda_token_account.clone()
                ],
                &[&[
                    exchange_wallet.key.as_ref(),
                    pda_seed,
                    &[bump_seed]]
                ],
            )?;
            msg!("PDA created for new token");
        }

        msg!["deposit tokens to program derived account"];
        let deposit_ix = &token_instruction::transfer(
            token_program.key,
            exchange_token_account.key,
            pda_token_account.key,
            exchange_wallet.key,
            &[exchange_wallet.key],
            amount
        )?;
        invoke(
            deposit_ix,
            &[
                mint.clone(),
                exchange_token_account.clone(),
                pda_token_account.clone(),
                exchange_wallet.clone(),
                exchange_program_account.clone(),
                token_program.clone()
            ]
        )?;

        Ok(())
    }

    fn withdraw(
        &mut self,
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
        pda_seed: &[u8;16],
        bump_seed: u8
    ) -> ProgramResult {

        let accounts_iter = &mut accounts.iter();

        let token_program = next_account_info(accounts_iter)?;
        let mint = next_account_info(accounts_iter)?;
        let exchange_wallet = next_account_info(accounts_iter)?;
        let exchange_token_account = next_account_info(accounts_iter)?;
        let pda_token_account = next_account_info(accounts_iter)?;
        let exchange_program_account = next_account_info(accounts_iter)?;

        if !exchange_wallet.is_signer {
            msg!("Missing required signature");
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mint_key = mint.key.to_string();
        let key = mint_key.as_str();
        if !self.token_mints.contains(key) {
            msg!("No such mint account");
            return Err(TokenError::MintMismatch.into());
        }

        msg!["withdraw tokens to owner (exchange) account"];
        let withdraw_ix = &token_instruction::transfer(
            token_program.key,
            pda_token_account.key,
            exchange_token_account.key,
            exchange_program_account.key,
            &[],
            amount
        )?;
        invoke_signed(
            withdraw_ix,
            &[
                mint.clone(),
                pda_token_account.clone(),
                exchange_token_account.clone(),
                exchange_program_account.clone(),
                exchange_wallet.clone(),
                token_program.clone()
            ],
            &[&[
                exchange_wallet.key.as_ref(),
                pda_seed,
                &[bump_seed]]
            ],
        )?;

        Ok(())
    }

    #[allow(non_snake_case)]
    fn exchange_sol_to_token(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64
    ) -> ProgramResult {

    let accounts_iter = &mut accounts.iter();
    let token_program = next_account_info(accounts_iter)?;
    let exchange_program_account = next_account_info(accounts_iter)?;
    let mint_A = next_account_info(accounts_iter)?;
    let mint_B = next_account_info(accounts_iter)?;
    let client_wallet = next_account_info(accounts_iter)?;
    let client_token_A_account = next_account_info(accounts_iter)?;
    let client_associated_token_B_account = next_account_info(accounts_iter)?;
    let exchange_wallet = next_account_info(accounts_iter)?;
    let exchange_token_A_account = next_account_info(accounts_iter)?;
    let exchange_associated_token_B_account = next_account_info(accounts_iter)?;
    // аккаунты, хранящие данные по стоимости token A, token B
    let token_A_data_feed_account = next_account_info(accounts_iter)?;
    let token_B_data_feed_account = next_account_info(accounts_iter)?;
    // программа, взаимодействующая с oracles
    let chainlink_program = next_account_info(accounts_iter)?;
    let token_A_data = Self::get_token_data(
        &chainlink_program.clone(),
        &token_A_data_feed_account.clone()
    )?;
    let token_B_data = Self::get_token_data(
        &chainlink_program.clone(),
        &token_B_data_feed_account.clone()
    )?;

    //let lamport_price = token_A_data.price / f64::from_u64(LAMPORTS_PER_SOL).unwrap();
    let token_A_amount = f64::from_u64(amount).unwrap() / LAMPORTS_PER_SOL as f64;
    let token_B_amount = (token_A_amount * token_A_data.price) / token_B_data.price;
    msg!["exchanging tokens : token A: {}, amount = {}, token B : {}, amount = {}",
        &token_A_data.description,
        &token_A_amount ,
        &token_B_data.description,
        &token_B_amount];

    let from_client_ix = &token_instruction::transfer(
        token_program.key,
        client_token_A_account.key,
        exchange_token_A_account.key,
        client_wallet.key,
        &[client_wallet.key, exchange_wallet.key],
        amount
    )?;
    invoke(
        from_client_ix,
        &[
            mint_A.clone(),
            client_token_A_account.clone(),
            exchange_token_A_account.clone(),
            client_wallet.clone(),
            exchange_wallet.clone(),
            token_program.clone(),
        ]
    )?;

    let to_client_ix = &token_instruction::transfer(
        token_program.key,
        exchange_associated_token_B_account.key,
        client_associated_token_B_account.key,
        exchange_wallet.key,
        &[exchange_wallet.key, client_wallet.key],
        (token_B_amount * (10.pow(token_B_data.decimals as u32) as f64)) as u64
    )?;
    invoke(to_client_ix,
                  &[
                      mint_B.clone(),
                      exchange_associated_token_B_account.clone(),
                      client_associated_token_B_account.clone(),
                      exchange_wallet.clone(),
                      client_wallet.clone(),
                      token_program.clone()
                  ]
        )?;

    Ok(())
    }

    #[allow(non_snake_case)]
    fn exchange_token_to_sol(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64
    ) -> ProgramResult {

        let accounts_iter = &mut accounts.iter();
        let token_program = next_account_info(accounts_iter)?;
        let exchange_program_account = next_account_info(accounts_iter)?;
        let mint_A = next_account_info(accounts_iter)?;
        let mint_B = next_account_info(accounts_iter)?;
        let client_wallet = next_account_info(accounts_iter)?;
        let client_associated_token_A_account = next_account_info(accounts_iter)?;
        let client_token_B_account = next_account_info(accounts_iter)?;
        let exchange_wallet = next_account_info(accounts_iter)?;
        let exchange_associated_token_A_account = next_account_info(accounts_iter)?;
        let exchange_token_B_account = next_account_info(accounts_iter)?;
        // аккаунты, хранящие данные по стоимости token A, token B
        let token_A_data_feed_account = next_account_info(accounts_iter)?;
        let token_B_data_feed_account = next_account_info(accounts_iter)?;
        // программа, взаимодействующая с oracles
        let chainlink_program = next_account_info(accounts_iter)?;
        let token_A_data = Self::get_token_data(
            &chainlink_program.clone(),
            &token_A_data_feed_account.clone()
        )?;
        let token_B_data = Self::get_token_data(
            &chainlink_program.clone(),
            &token_B_data_feed_account.clone()
        )?;

        let lamport_price = token_B_data.price / f64::from_u64(LAMPORTS_PER_SOL).unwrap();
        let token_A_amount = amount / (10.pow(token_A_data.decimals as u32) as u64);
        let token_B_amount = ((f64::from_u64(amount).unwrap()) * token_A_data.price / token_B_data.price * lamport_price) as u64;
        msg!["exchanging tokens : token A: {}, amount={}, token B : {}, amount={}",
            &token_A_data.description,
            &token_A_amount,
            &token_B_data.description,
            &token_B_amount / LAMPORTS_PER_SOL];

        let from_client_ix = &token_instruction::transfer(
            token_program.key,
            client_associated_token_A_account.key,
            exchange_associated_token_A_account.key,
            client_wallet.key,
            &[client_wallet.key, exchange_wallet.key],
            token_A_amount
        )?;
        invoke(
            from_client_ix,
            &[
                mint_A.clone(),
                client_associated_token_A_account.clone(),
                exchange_associated_token_A_account.clone(),
                client_wallet.clone(),
                exchange_wallet.clone(),
                token_program.clone(),
            ]
        )?;
        let to_client_ix = &token_instruction::transfer(
            token_program.key,
            exchange_token_B_account.key,
            client_token_B_account.key,
            exchange_wallet.key,
            &[exchange_wallet.key, client_wallet.key],
            token_B_amount
        )?;
        invoke(to_client_ix,
                      &[
                          mint_B.clone(),
                          exchange_token_B_account.clone(),
                          client_token_B_account.clone(),
                          exchange_wallet.clone(),
                          client_wallet.clone(),
                          token_program.clone()
                      ]
        )?;
        Ok(())
    }

    #[allow(non_snake_case)]
    fn exchange_token_to_token(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64
    ) -> ProgramResult {

        let accounts_iter = &mut accounts.iter();
        let token_program = next_account_info(accounts_iter)?;
        let exchange_program_account = next_account_info(accounts_iter)?;
        let mint_A = next_account_info(accounts_iter)?;
        let mint_B = next_account_info(accounts_iter)?;
        let client_wallet = next_account_info(accounts_iter)?;
        let client_associated_token_A_account = next_account_info(accounts_iter)?;
        let client_associated_token_B_account = next_account_info(accounts_iter)?;
        let exchange_wallet = next_account_info(accounts_iter)?;
        let exchange_associated_token_A_account = next_account_info(accounts_iter)?;
        let exchange_associated_token_B_account = next_account_info(accounts_iter)?;
        // аккаунты, хранящие данные по стоимости token A, token B
        let token_A_data_feed_account = next_account_info(accounts_iter)?;
        let token_B_data_feed_account = next_account_info(accounts_iter)?;
        // программа, взаимодействующая с oracles
        let chainlink_program = next_account_info(accounts_iter)?;
        let token_A_data = Self::get_token_data(
            &chainlink_program.clone(),
            &token_A_data_feed_account.clone()
        )?;
        let token_B_data = Self::get_token_data(
            &chainlink_program.clone(),
            &token_B_data_feed_account.clone()
        )?;

        let token_A_amount = amount;
        let token_B_amount = ((f64::from_u64(amount).unwrap()) / token_B_data.price) as u64;
        msg!["exchanging tokens : token A: {}, amount= {}, token B : {}, amount={}",
            &token_A_data.description,
            &token_A_amount,
            &token_B_data.description,
            &token_B_amount];

        let from_client_ix = &token_instruction::transfer(
            token_program.key,
            client_associated_token_A_account.key,
            exchange_associated_token_A_account.key,
            client_wallet.key,
            &[client_wallet.key, exchange_wallet.key],
            token_A_amount
        )?;
        invoke(
            from_client_ix,
            &[
                mint_A.clone(),
                client_associated_token_A_account.clone(),
                exchange_associated_token_A_account.clone(),
                client_wallet.clone(),
                exchange_wallet.clone(),
                token_program.clone(),
            ]
        )?;

        let (_, bump_seed) = Pubkey::find_program_address(
            &[
                &exchange_wallet.key.to_bytes(),
                &token_program.key.to_bytes(),
                &mint_A.key.to_bytes(),
                &mint_B.key.to_bytes(),
            ],
            program_id
        );

        let to_client_ix = &token_instruction::transfer(
            token_program.key,
            exchange_associated_token_B_account.key,
            client_associated_token_B_account.key,
            exchange_program_account.key,
            &[],
            token_B_amount
        )?;
        invoke_signed(to_client_ix,
                      &[
                          exchange_associated_token_B_account.clone(),
                          client_associated_token_B_account.clone(),
                          exchange_program_account.clone(),
                          token_program.clone()
                      ],
                      &[&[
                          &exchange_wallet.key.to_bytes(),
                          &token_program.key.to_bytes(),
                          &mint_A.key.to_bytes(),
                          &mint_B.key.to_bytes(),
                          &[bump_seed]
                      ]]
        )?;
        Ok(())
    }

    fn get_token_data<'a>(
        chainlink_program: &AccountInfo<'a>,
        data_feed_program: &AccountInfo<'a>)
        -> Result<TokenData, ProgramError>
    {
        let oracle_data = {
            let round = chainlink::latest_round_data(
                chainlink_program.clone(),
                data_feed_program.clone(),
            );
            let decimals = chainlink::decimals(
                chainlink_program.clone(),
                data_feed_program.clone(),
            );
            let token_description = chainlink::description(
                chainlink_program.clone(),
                data_feed_program.clone());
            match (round, decimals, token_description) {
                (Ok(round), Ok(decimals), Ok(token_description)) => {
                    Ok((round, decimals, token_description))
                }
                _ => { Err(ProgramError::from(TokenError::OracleDataFeedError)) }
            }
        };
        if let Ok(oracle_data) = oracle_data {
            let price = f64::from_i128(oracle_data.0.answer).unwrap() /
                f64::from_i128(10_i128.pow(oracle_data.1 as u32)).unwrap();
            let trim_usd = oracle_data.2.len() - 6;
            let (token, _) = oracle_data.2.split_at(trim_usd);
            msg!("price of token {} is {} USD", &token , price);
            Ok(
                TokenData {
                price,
                decimals: oracle_data.1,
                description: String::from(token),
            })
        } else {
            Err(ProgramError::from(TokenError::OracleDataFeedError))
        }
    }
}

struct TokenData {
    price: f64,
    decimals: u8,
    description: String,
}
