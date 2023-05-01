use std::fmt::{Display, Formatter};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    native_token::LAMPORTS_PER_SOL,
    program:: invoke,
    program_error::ProgramError,
    pubkey::Pubkey
};
use spl_token::{
    self,
    instruction as token_instruction,
};
use chainlink_solana as chainlink;
use num_traits::{FromPrimitive, Pow};
use solana_program::program::invoke_signed;
use solana_program::program_pack::Pack;
use spl_token::state::{Account, Mint};
use crate::error::TokenError;
use crate::instruction::ExchangeInstruction;


// TOKENS ARE ALWAYS TRANSFERRED :
// FROM A TO A (FROM CLIENT TO EXCHANGE)
// FROM B TO B (FROM EXCHANGE TO CLIENT)
#[allow(non_snake_case)]
pub fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {

    let accounts_iter = &mut accounts.iter();
    let token_program = next_account_info(accounts_iter)?;
    let mint_A = next_account_info(accounts_iter)?;
    let mint_B = next_account_info(accounts_iter)?;
    let client_wallet = next_account_info(accounts_iter)?;
    let exchange_wallet = next_account_info(accounts_iter)?;
    let instruction = ExchangeInstruction::unpack(instruction_data)?;
    // аккаунты, хранящие данные по стоимости token A, token B
    let token_A_data_feed_account = next_account_info(accounts_iter)?;
    let token_B_data_feed_account = next_account_info(accounts_iter)?;
    // программа, взаимодействующая с oracles
    let chainlink_program = next_account_info(accounts_iter)?;
    let token_A = get_token_price_and_description(
        &chainlink_program.clone(),
        &token_A_data_feed_account.clone()
    )?;
    let token_B = get_token_price_and_description(
        &chainlink_program.clone(),
        &token_B_data_feed_account.clone()
    )?;

    let mut client_token_A_account = None;
    let mut client_associated_token_A_account = None;
    let mut client_token_B_account = None;
    let mut client_associated_token_B_account = None;
    let mut exchange_token_A_account = None;
    let mut exchange_associated_token_A_account = None;
    let mut exchange_token_B_account = None;
    let mut exchange_associated_token_B_account = None;

    match instruction {

        ExchangeInstruction::SolToToken { .. } => {

            let client_accounts = Accounts {
                wallet: client_wallet,
                token_A_account: client_token_A_account,
                token_A_associated_account: client_associated_token_A_account,
                token_B_account: client_token_B_account,
                token_B_associated_account: client_associated_token_B_account,
            };

            let exchange_accounts = Accounts {
                wallet: exchange_wallet,
                token_A_account: exchange_token_A_account,
                token_A_associated_account: exchange_associated_token_A_account,
                token_B_account: exchange_token_B_account,
                token_B_associated_account: exchange_associated_token_B_account,
            };

            client_token_A_account = Some(next_account_info(accounts_iter)?);
            msg!("client_token_A_account : {}", client_token_A_account.unwrap().key);
            client_associated_token_B_account = Some(next_account_info(accounts_iter)?);
            msg!("client_associated_token_B_account : {}", client_associated_token_B_account.unwrap().key);
            exchange_token_A_account = Some(next_account_info(accounts_iter)?);
            msg!("exchange_token_A_account : {}", exchange_token_A_account.unwrap().key);
            exchange_associated_token_B_account = Some(next_account_info(accounts_iter)?);
            msg!("exchange_associated_token_B_account : {}", exchange_associated_token_B_account.unwrap().key);


            let from_client_ix = &token_instruction::transfer(
                token_program.key,
                client_token_A_account.unwrap().key,
                exchange_token_A_account.unwrap().key,
                client_wallet.key,
                &[client_wallet.key, exchange_wallet.key],
                50000000
            )?;
            invoke(
                from_client_ix,
                &[
                    mint_A.clone(),
                    client_token_A_account.unwrap().clone(),
                    exchange_token_A_account.unwrap().clone(),
                    client_wallet.clone(),
                    exchange_wallet.clone(),
                    token_program.clone(),
                ]
            )?;

            let exchange_associated_token_B_account = exchange_associated_token_B_account.unwrap();
            let exchange_program_pda = next_account_info(accounts_iter)?;
            msg!["exchange_program_pda key: {}", exchange_program_pda.key];

            let (exchange_associated_token_address, bump_seed) = Pubkey::find_program_address(
                &[
                    &exchange_wallet.key.to_bytes(),
                    &token_program.key.to_bytes(),
                    &mint_B.key.to_bytes(),
                ],
                program_id
            );

            msg!["exchange_associated_token_address, bump_seed : {}, {}", &exchange_associated_token_address, &bump_seed];

            let to_client_ix = &token_instruction::transfer(
                token_program.key,
                exchange_associated_token_B_account.key,  // &exchange_associated_token_address,
                client_associated_token_B_account.unwrap().key,
                exchange_program_pda.key,
                &[],
                1000
            )?;

            invoke_signed(to_client_ix,
                          &[
                              exchange_associated_token_B_account.clone(),
                              client_associated_token_B_account.unwrap().clone(),
                              exchange_program_pda.clone(),
                              token_program.clone()
                          ],
                          &[&[
                              &exchange_wallet.key.to_bytes(),
                              &token_program.key.to_bytes(),
                              &mint_B.key.to_bytes(),
                              &[bump_seed]
                          ]]
            )?;

            // exchange(
            //     token_program,
            //     mint_A,
            //     mint_B,
            //     client_accounts,
            //     exchange_accounts,
            //     token_A,
            //     token_B,
            //     instruction
            // )?;
        }

        ExchangeInstruction::TokenToSol { .. } => {

            client_associated_token_A_account = Some(next_account_info(accounts_iter)?);   // 1
            client_token_B_account = Some(next_account_info(accounts_iter)?);              // 4
            exchange_associated_token_A_account = Some(next_account_info(accounts_iter)?); // 2
            exchange_token_B_account = Some(next_account_info(accounts_iter)?);            // 3

            let client_accounts = Accounts {
                wallet: client_wallet,
                token_A_account: client_token_A_account,
                token_A_associated_account: client_associated_token_A_account,
                token_B_account: client_token_B_account,
                token_B_associated_account: client_associated_token_B_account,
            };

            let exchange_accounts = Accounts {
                wallet: exchange_wallet,
                token_A_account: exchange_token_A_account,
                token_A_associated_account: exchange_associated_token_A_account,
                token_B_account: exchange_token_B_account,
                token_B_associated_account: exchange_associated_token_B_account,
            };

            exchange(
                token_program,
                mint_A,
                mint_B,
                client_accounts,
                exchange_accounts,
                token_A,
                token_B,
                instruction
            )?;
        }

        ExchangeInstruction::TokenToToken { .. } => {

            client_associated_token_A_account = Some(next_account_info(accounts_iter)?);   // 1
            client_associated_token_B_account = Some(next_account_info(accounts_iter)?);   // 4
            exchange_associated_token_A_account = Some(next_account_info(accounts_iter)?); // 2
            exchange_associated_token_B_account = Some(next_account_info(accounts_iter)?); // 3

            let client_accounts = Accounts {
                wallet: client_wallet,
                token_A_account: client_token_A_account,
                token_A_associated_account: client_associated_token_A_account,
                token_B_account: client_token_B_account,
                token_B_associated_account: client_associated_token_B_account,
            };

            let exchange_accounts = Accounts {
                wallet: exchange_wallet,
                token_A_account: exchange_token_A_account,
                token_A_associated_account: exchange_associated_token_A_account,
                token_B_account: exchange_token_B_account,
                token_B_associated_account: exchange_associated_token_B_account,
            };

            exchange(
                token_program,
                mint_A,
                mint_B,
                client_accounts,
                exchange_accounts,
                token_A,
                token_B,
                instruction
            )?;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[allow(non_snake_case)]
fn exchange<'a> (
    token_program: &AccountInfo<'a>,
    mint_A: &AccountInfo<'a>,
    mint_B: &AccountInfo<'a>,
    client_accounts: Accounts<'a>,
    exchange_accounts: Accounts<'a>,
    token_A: (f64, String), // ( price, token name )
    token_B: (f64, String), // ( price, token name )
    instruction: ExchangeInstruction
)
    -> Result<(), ProgramError> {

    msg!("mint_A : {}", mint_A.key);
    msg!("mint_B : {}", mint_B.key);
    msg!("client_accounts : {}", client_accounts.to_string());
    msg!("exchange_accounts : {}", exchange_accounts.to_string());

    let token_A_price = token_A.0;
    let token_A_description = token_A.1;
    let token_B_price = token_B.0;
    let token_B_description = token_B.1;
    let token_A_amount;
    let token_B_amount;

    let mut client_associated_token_A_account = None;
    let mut client_associated_token_B_account = None;
    let mut exchange_associated_token_A_account = None;
    let mut exchange_associated_token_B_account = None;

    match instruction {
        ExchangeInstruction::SolToToken { amount} => {
            let lamport_price = token_A_price / f64::from_u64(LAMPORTS_PER_SOL).unwrap();
            token_A_amount = amount;
            token_B_amount = ((f64::from_u64(amount).unwrap() * lamport_price)
                                                    / token_B_price) as u64;
            client_associated_token_B_account = client_accounts.token_B_associated_account;
            exchange_associated_token_B_account = exchange_accounts.token_B_associated_account;
        }
        ExchangeInstruction::TokenToSol { amount } => {
            let lamport_price = token_B_price / f64::from_u64(LAMPORTS_PER_SOL).unwrap();
            token_A_amount = amount;
            token_B_amount = ((f64::from_u64(amount).unwrap())
                / token_B_price * lamport_price) as u64;
            client_associated_token_A_account = client_accounts.token_A_associated_account;
            exchange_associated_token_A_account = exchange_accounts.token_A_associated_account;
        }
        ExchangeInstruction::TokenToToken { amount } => {
            token_A_amount = amount;
            token_B_amount = ((f64::from_u64(amount).unwrap())
                / token_B_price) as u64;
            client_associated_token_A_account = client_accounts.token_A_associated_account;
            client_associated_token_B_account = client_accounts.token_B_associated_account;
            exchange_associated_token_A_account = exchange_accounts.token_A_associated_account;
            exchange_associated_token_B_account = exchange_accounts.token_B_associated_account;
        }
    }
    msg!("exchange {token_A_description} to {token_B_description}...\
                   {token_A_description} amount: {}, {token_B_description} amount: {}",
                   token_A_amount, token_B_amount);
    // validate_transaction(
    //     client_wallet,
    //     client_sol_account,
    //     exchange_usdc_account,
    //     usdc_amount
    // )?;

    transfer_tokens(
        token_program,
        mint_A,
        mint_B,
        client_accounts.wallet,
        client_accounts.token_A_account,
        client_accounts.token_B_account,
        client_associated_token_A_account,
        client_associated_token_B_account,
        exchange_accounts.wallet,
        exchange_accounts.token_A_account,
        exchange_accounts.token_B_account,
        exchange_associated_token_A_account,
        exchange_associated_token_B_account,
        token_A_amount,
        token_B_amount
    )?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[allow(non_snake_case)]
fn transfer_tokens<'a> (
    token_program: &AccountInfo<'a>,
    source_mint: &AccountInfo<'a>,
    destination_mint: &AccountInfo<'a>,
    client_wallet: &AccountInfo<'a>,
    client_token_A_account: Option<&AccountInfo<'a>>,
    client_token_B_account: Option<&AccountInfo<'a>>,
    client_associated_token_A_account: Option<&AccountInfo<'a>>,
    client_associated_token_B_account: Option<&AccountInfo<'a>>,
    exchange_wallet: &AccountInfo<'a>,
    exchange_token_A_account: Option<&AccountInfo<'a>>,
    exchange_token_B_account: Option<&AccountInfo<'a>>,
    exchange_associated_token_A_account: Option<&AccountInfo<'a>>,
    exchange_associated_token_B_account: Option<&AccountInfo<'a>>,
    source_amount: u64,
    destination_amount: u64
) -> Result<(), ProgramError> 
{
    msg!["transfer_tokens()..."];
    // В зависимости от того, куда переводим, потребуются разные аккаунты
    //    1)  Sol -> Token    Token -> Sol
    //    2)  Token -> Sol    Sol   -> Token
    //    3)  Token -> Token  Token -> Token
    let from_client_to_exchange_sender;      // 1
    let to_exchange_from_client_recipient;   // 2
    let from_exchange_to_client_sender;      // 3
    let to_client_from_exchange_recipient;   // 4
    match (client_token_A_account, client_associated_token_A_account, client_token_B_account, client_associated_token_B_account,
           exchange_token_A_account, exchange_associated_token_A_account, exchange_token_B_account, exchange_associated_token_B_account)
    {
        (Some(c_t_A_a), None, None, Some(c_a_t_B_a), Some(e_t_A_a), None, None, Some(e_a_t_B_a)) =>
            {
                from_client_to_exchange_sender = c_t_A_a;
                to_exchange_from_client_recipient = e_t_A_a;
                from_exchange_to_client_sender = e_a_t_B_a;
                to_client_from_exchange_recipient = c_a_t_B_a;
            },
        (None, Some(c_a_t_A_a), Some(c_t_B_a), None, None, Some(e_a_t_A_a), Some(e_t_B_a), None) =>
            {
                from_client_to_exchange_sender = c_a_t_A_a;
                to_exchange_from_client_recipient = e_a_t_A_a;
                from_exchange_to_client_sender = e_t_B_a;
                to_client_from_exchange_recipient = c_t_B_a;
            },
        (None, Some(c_a_t_A_a), None, Some(c_a_t_B_a), None, Some(e_a_t_A_a), None, Some(e_a_t_B_a)) =>
            {
                from_client_to_exchange_sender = c_a_t_A_a;
                to_exchange_from_client_recipient = e_a_t_A_a;
                from_exchange_to_client_sender = e_a_t_B_a;
                to_client_from_exchange_recipient = c_a_t_B_a;
            }
        _ => { return Err(ProgramError::from(TokenError::MismatchedAccountsError)) }
    }

    msg!("
       from_client_to_exchange_sender = {};
       to_exchange_from_client_recipient = {};
       from_exchange_to_client_sender = {};
       to_client_from_exchange_recipient = {};
    ", from_client_to_exchange_sender.key,
       to_exchange_from_client_recipient.key,
       from_exchange_to_client_sender.key,
       to_client_from_exchange_recipient.key
    );
    // отправляем tokens с адреса отправителя на адрес exchange
    let from_client_ix = &token_instruction::transfer(
        token_program.key,
        from_client_to_exchange_sender.key,
        to_exchange_from_client_recipient.key,
        client_wallet.key,
        &[client_wallet.key, exchange_wallet.key],
        source_amount
    )?;
    invoke(
        from_client_ix,
        &[
            source_mint.clone(),
            from_client_to_exchange_sender.clone(),
            to_exchange_from_client_recipient.clone(),
            client_wallet.clone(),
            exchange_wallet.clone(),
            token_program.clone(),
        ]
    )?;
    // Отправляем токены на адрес получателя
    let to_client_ix = &token_instruction::transfer(
        token_program.key,
        from_exchange_to_client_sender.key,
        to_client_from_exchange_recipient.key,
        exchange_wallet.key,
        &[exchange_wallet.key, client_wallet.key],
        destination_amount
    )?;
    invoke(
        to_client_ix,
        &[
            destination_mint.clone(),
            from_exchange_to_client_sender.clone(),
            to_client_from_exchange_recipient.clone(),
            exchange_wallet.clone(),
            client_wallet.clone(),
            token_program.clone()
        ]
    )?;

    Ok(()) 
}

#[allow(non_snake_case)]
struct Accounts<'a> {
    wallet: &'a AccountInfo<'a>,
    token_A_account: Option<&'a AccountInfo<'a>>,             // for Wrapped Sol
    token_A_associated_account: Option<&'a AccountInfo<'a>>,  // for any tokens
    token_B_account: Option<&'a AccountInfo<'a>>,             // for Wrapped Sol
    token_B_associated_account: Option<&'a AccountInfo<'a>>   // for any tokens
}

impl Display for Accounts<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let accounts = [self.token_A_account, self.token_A_associated_account, self.token_B_account, self.token_B_associated_account]
            .iter()
            .map(|account_key_opt| {
                let mut token_account_key = String::from("");
                if let Some(token) = account_key_opt {
                    token_account_key += token.key.to_string().as_str();
                } else {
                    token_account_key += "_";
                }
                token_account_key
            })
            .collect::<Vec<String>>();

        write!(
            f,
            "wallet: {},
            token_A_account: {},
            token_A_associated account: {},
            token_B_account: {},
            token_B_associated account: {}",
            self.wallet.key.to_string().as_str(),
            accounts.get(0).unwrap(),
            accounts.get(1).unwrap(),
            accounts.get(2).unwrap(),
            accounts.get(3).unwrap()
        )
    }
}

fn get_token_price_and_description<'a>(
    chainlink_program: &AccountInfo<'a>,
    data_feed_program: &AccountInfo<'a>)
    -> Result<(f64, String), ProgramError>
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
        msg!("price of token {} is {}", token , price);
        Ok ((price, String::from(token)))
    } else {
        Err(ProgramError::from(TokenError::OracleDataFeedError))
    }
}

// fn validate_transaction<'a> (
//     client_wallet: &AccountInfo<'a>,
//     client_account: &AccountInfo<'a>,
//     exchange_account: &AccountInfo<'a>,
//     amount: u64,
// ) -> Result<(), ProgramError>  {
//     // Проверяем, что client аккаунт принадлежит отправителю
//     // if client_account.owner != client_wallet.key {
//     //     msg!("client_account.owner: {} ", &client_account.owner);
//     //     msg!("client_wallet.key: {} ", &client_wallet.key);
//     //     msg!("Invalid account data: source_account.owner != owner_account PubKey");
//     //     return Err(ProgramError::InvalidAccountData);
//     // }
//     // Проверяем, что у client аккаунта достаточно токенов для обмена
//     let client_account_to_exchange =
//         TokenAccount::unpack(&client_account.data.borrow())?;
//     let client_balance = client_account_to_exchange.amount;
//     //msg!("client_balance: {}", &client_balance);
//     if amount > client_balance {
//         msg!("Insufficient funds");
//         return Err(ProgramError::InsufficientFunds);
//     }
//     // Проверяем, что владелец аккаунта подписал транзакцию
//     // if !client_wallet.is_signer {
//     //     msg!("Missing required signature");
//     //     return Err(ProgramError::MissingRequiredSignature);
//     // }
//     // Проверяем, что на аккаунте exchange достаточно tokens для обмена
//     let token_exchange_account =
//         TokenAccount::unpack(&exchange_account.data.borrow())?;
//     let exchange_balance = token_exchange_account.amount;
//     if exchange_balance < amount {
//         msg!("not enough balance to exchange: {}, available amount is only {}",
//                                                amount, exchange_balance);
//         return Err(ProgramError::from(TokenError::NotEnoughBalanceToExchange));
//     };
//
//     Ok(())
// }

// // let mut exchange_token_B_acc =
//             //     StateWithExtensionsMut::<Account>::unpack(&mut exchange_token_B_account_data)?;
//
//             // let approve_from_exchange_ix = &token_instruction::approve(
//             //     token_program.key,
//             //     exchange_token_B_account.key,
//             //     exchange_wallet.key,
//             //     exchange_associated_token_B_account.key,
//             //     &[exchange_wallet.key],
//             //     1
//             // )?;
//             // invoke(approve_from_exchange_ix, &[
//             //     exchange_token_B_account.clone(),
//             //     exchange_wallet.clone(),
//             //     exchange_associated_token_B_account.clone()
//             // ]
//             //               // ,
//             //               // &[
//             //               //     &[
//             //               //         &exchange_wallet.key.to_bytes(),
//             //               //         &token_program.key.to_bytes(),
//             //               //         &mint_B.key.to_bytes()
//             //               //     ]
//             //               // ]
//             // )?;
