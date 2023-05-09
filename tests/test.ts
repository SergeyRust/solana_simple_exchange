import {
    clusterApiUrl, ComputeBudgetProgram,
    Connection,
    Keypair,
    LAMPORTS_PER_SOL,
    PublicKey, sendAndConfirmTransaction,
    Transaction,
    TransactionInstruction, VersionedTransaction,
}
    from '@solana/web3.js';

import {
    approveChecked,
    createAccount, createApproveCheckedInstruction, createAssociatedTokenAccount,
    getAccount, getAssociatedTokenAddress,
    getOrCreateAssociatedTokenAccount,
    NATIVE_MINT,
    TOKEN_PROGRAM_ID
}
    from '@solana/spl-token';
import * as token from '@solana/spl-token'

import * as web3 from "@solana/web3.js";
import * as borsh from '@project-serum/borsh'
import {findProgramAddressSync} from "@project-serum/anchor/dist/cjs/utils/pubkey";

const anchor = require('@project-serum/anchor')

async function getBalanceUsingWeb3(address: PublicKey, connection: Connection): Promise<number> {
    return connection.getBalance(address);
}

function createKeypairFromFile(path: string): Keypair {
    return Keypair.fromSecretKey(
        Buffer.from(JSON.parse(require('fs').readFileSync(path, "utf-8")))
    )
}

const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID: PublicKey = new PublicKey(
    'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL',
);

function findAssociatedTokenAddress(
    walletAddress: PublicKey,
    tokenMintAddress: PublicKey
): PublicKey {
    return PublicKey.findProgramAddressSync(
        [
            walletAddress.toBuffer(),
            TOKEN_PROGRAM_ID.toBuffer(),
            tokenMintAddress.toBuffer(),
        ],
        SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID
    )[0];
}

describe("Test1", async () => {

    const connection = new Connection(clusterApiUrl('devnet'));

    const payer = createKeypairFromFile(require('os').homedir() + '/.config/solana/id.json');
    //const program = createKeypairFromFile('./program/target/deploy/bridge_contract_usdc_sol-keypair.json');

    const upgradeAuthorityOfBridgeProgram = new PublicKey(
        "vayy8SrMqXogS5qqeKxqLFcXUvjDASoDV9knnnSYTfk"
    );
    const token_program_id = new PublicKey(
        //"TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb" //token2022
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    );
    const USDC_MINT = new PublicKey(
        "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"
    );

    const CUSTOM_MINT = new PublicKey(
        "BLnd4ysyGjCftybxYH1huZvZaRsC17DTvjZFFsXgaM9K"
    );

    const exchangeProgram: PublicKey = new PublicKey(
        "G8vMwzB6DXD7E3zz4Xwm9x2JcZsiR7zhiKKimiMeFarS",
    );
    const bridgeProgramCustomAssociatedTokenAccount = new PublicKey(
        "D2FskqMUDcLmQVX5SHRhJ5PJy6Ntr8zZr89J9Ufd6vGS" // "D2FskqMUDcLmQVX5SHRhJ5PJy6Ntr8zZr89J9Ufd6vGS" - first
    );
    const system_program_id = new PublicKey(
        "11111111111111111111111111111111"
    );
    // const clientWallet = new PublicKey(
    //     "8kzvhvtMbRv36zgBp7ezfEZb8kL2usQpjHXDBjbGh83e"
    // );
    const clientWallet = createKeypairFromFile(require('os').homedir() + '/my-solana-wallet/client1.json');
    const clientWrappedSolAccount = new PublicKey(
        "CsrhZBzHFwbn1LcAQDgvBE2KxPpgeQwj2Ur22qPrewFx"
    );

    const clientUsdcAssociatedTokenAccount = new PublicKey(
        "Aa9WvdY9werjTsJTQSGvqqaS6WYSoRf56Wqap39zAKWz"
    );
    const clientCustomAssociatedTokenAccount = new PublicKey(
        "AJc7EGu6pcN62sh1QmDkp4qqUMSEk6kMmLbq8PmHFZAP"
    );


    // const exchangeWallet = new PublicKey(
    //   "8CuWXnVSPR83xJeF5oZzwLEQamgx1YoRWsjvDRwD3GBP" //
    // );
    const exchangeWallet = createKeypairFromFile(require('os').homedir() + '/my-solana-wallet/exchange/wallet1.json');
    const exchangeWrappedSolAccount = new PublicKey(
        "HNeSrHNCXf4YFZJbaRBMWpK8pfQ1xogDUXon3grjzE8D"
    );
    const exchangeUsdcAssociatedTokenAccount = new PublicKey(
        "Dmr6wkEHhy7dFQN62C3NyiEuDkw21kFUBPwKeFVxc4qs"
    );

    const exchangeCustomAssociatedTokenAccount = new PublicKey(
        "8T3mAnbgMvXocoYneuWJgDnntMMQPDdHDcmSKqwcXtte"
    );

    const usdcToUsdDataFeedAccount = new PublicKey(
        "2EmfL3MqL3YHABudGNmajjCpR13NNEn9Y4LWxbDm6SwR"
    );
    const solToUsdDataFeedAccount = new PublicKey(
        "99B2bTijsU6f1GCT73HmdR7HCFFjGMBcPZY6jZ96ynrR"
    );
    const chainLinkProgramId = new PublicKey(
        "HEvSKofvBgfaexv23kMabbYqxasxU3mQ4ibBMEmJWHny",
    );

    it("Deposit tokens", async() => {

        const exchangePda = PublicKey.findProgramAddressSync(
            [
                Buffer.from( "toKeNpDaSeEd"),
                //exchangeProgram.toBuffer(),
                CUSTOM_MINT.toBuffer(),

            ],
            exchangeProgram
        );
        console.log("exchangePda pubkey: %s ", exchangePda.at(0));
        let pdaAccount = new PublicKey(
            exchangePda.at(0));

        const instructionData = borsh.struct([
            borsh.u8('variant'),
            borsh.u64('amount'),
            borsh.str('pda_seed'),
            borsh.u8('bump_seed')
        ])
        const buffer = Buffer.alloc(1000);
        const value = new anchor.BN(5);
        instructionData.encode({
            variant: 0,
            amount : value,
            pda_seed: String("toKeNpDaSeEd"),
            bump_seed: exchangePda.at(1)
        }, buffer);
        const instructionBuffer = buffer.slice(0, instructionData.getSpan(buffer));

        let ix = new TransactionInstruction({
            keys: [
                {pubkey: token_program_id, isSigner: false, isWritable: false},
                {pubkey: CUSTOM_MINT, isSigner: false, isWritable: false},
                {pubkey: exchangeWallet.publicKey, isSigner: true, isWritable: true},
                {pubkey: exchangeCustomAssociatedTokenAccount, isSigner: false, isWritable: true},
                {pubkey: pdaAccount, isSigner: false , isWritable: true},
                {pubkey: exchangeProgram, isSigner: true, isWritable: true},
            ],
            data: instructionBuffer,
            programId: exchangeProgram,
        });

            // const modifyComputeUnits = ComputeBudgetProgram.setComputeUnitLimit({
            //     units: 1000000
            // });
            let tx = new Transaction();
            //tx.add(modifyComputeUnits);
            tx.recentBlockhash = ( await connection.getLatestBlockhash('finalized')).blockhash;
            tx.feePayer = payer.publicKey;
            tx.add(ix);

            console.log(tx.serialize().toString("base64"));

            let sim_result = await connection.simulateTransaction(tx);
            console.log("logs : {}", sim_result.value.logs);

        //let minRent = await connection.getMinimumBalanceForRentExemption(0);
        // let blockhash = await connection
        //     .getLatestBlockhash()
        //     .then((res) => res.blockhash);
        // const messageV0 = new web3.TransactionMessage({
        //     instructions: [ix],
        //     payerKey: exchangeWallet.publicKey,
        //     recentBlockhash: blockhash,
        // }).compileToV0Message();
        //
        //
        // const transaction = new web3.VersionedTransaction(messageV0);
        // transaction.sign([exchangeWallet, ]);
        //
        //  let committed = await connection.sendTransaction(
        //         transaction,);
        //     console.log("transaction: {}", committed)
    });

    it("Exchange sol to USDC", async () => {

        const instructionData = borsh.struct([
            borsh.u8('variant'),
            borsh.u64('amount'),
            borsh.str('pda_seed'),
            borsh.u8('bump_seed')
        ])
        const buffer = Buffer.alloc(1000);
        const value = new anchor.BN(100000000);
        instructionData.encode(
            {
                variant: 2,
                amount : value
            },
            buffer);
        const instructionBuffer = buffer.slice(0, instructionData.getSpan(buffer));

        let ix = new TransactionInstruction({
            keys: [
                {pubkey: token_program_id, isSigner: false, isWritable: false},
                {pubkey: exchangeProgram, isSigner: false, isWritable: false},
                {pubkey: NATIVE_MINT, isSigner: false, isWritable: false},
                {pubkey: USDC_MINT, isSigner: false, isWritable: false},
                {pubkey: clientWallet.publicKey, isSigner: true, isWritable: true},
                {pubkey: clientWrappedSolAccount, isSigner: false, isWritable: true},
                {pubkey: clientUsdcAssociatedTokenAccount, isSigner: false, isWritable: true},
                {pubkey: exchangeWallet.publicKey, isSigner: true, isWritable: true},
                {pubkey: exchangeWrappedSolAccount, isSigner: false, isWritable: true},
                {pubkey: exchangeUsdcAssociatedTokenAccount, isSigner: false, isWritable: true},
                {pubkey: solToUsdDataFeedAccount, isSigner: false, isWritable: false},
                {pubkey: usdcToUsdDataFeedAccount, isSigner: false, isWritable: false},
                {pubkey: chainLinkProgramId, isSigner: false, isWritable: false},
            ],
            data: instructionBuffer,
            programId: exchangeProgram,
        });


        // let tx = new Transaction();
        // tx.feePayer = payer.publicKey;
        // tx.add(ix);
        //
        // let sim_result = await connection.simulateTransaction(tx);
        // console.log("logs : {}", sim_result.value.logs);

        let tx = new Transaction();
        tx.recentBlockhash = (await connection.getLatestBlockhash('finalized')).blockhash;
        tx.feePayer = clientWallet.publicKey;
        tx.add(ix);

        let committed = await sendAndConfirmTransaction(
            connection,
            tx,
            [clientWallet, exchangeWallet],
            {
                skipPreflight: true,
                preflightCommitment : "finalized"
            }
        );
        console.log("transaction: {}", committed)
    });

    it("Exchange USDC to sol", async () => {

        const instructionData = borsh.struct([
            borsh.u8('variant'),
            borsh.u64('amount'),
            borsh.str('pda_seed'),
            borsh.u8('bump_seed')
        ])
        const buffer = Buffer.alloc(1000);
        const value = new anchor.BN(5000000);  // 5 usdc
        instructionData.encode(
            {
                variant: 3,
                amount : value,
            },
            buffer);
        const instructionBuffer = buffer.slice(0, instructionData.getSpan(buffer));

        let ix = new TransactionInstruction({
            keys: [
                {pubkey: token_program_id, isSigner: false, isWritable: false},
                {pubkey: exchangeProgram, isSigner: true, isWritable: false},
                {pubkey: USDC_MINT, isSigner: false, isWritable: false},
                {pubkey: NATIVE_MINT, isSigner: false, isWritable: false},
                {pubkey: clientWallet.publicKey, isSigner: true, isWritable: true},
                {pubkey: clientUsdcAssociatedTokenAccount, isSigner: false, isWritable: true},
                {pubkey: clientWrappedSolAccount, isSigner: false, isWritable: true},
                {pubkey: exchangeWallet.publicKey, isSigner: true, isWritable: true},
                {pubkey: exchangeUsdcAssociatedTokenAccount, isSigner: false, isWritable: true},
                {pubkey: exchangeWrappedSolAccount, isSigner: false, isWritable: true},
                {pubkey: usdcToUsdDataFeedAccount, isSigner: false, isWritable: false},
                {pubkey: solToUsdDataFeedAccount, isSigner: false, isWritable: false},
                {pubkey: chainLinkProgramId, isSigner: false, isWritable: false},
            ],
            data: instructionBuffer,
            programId: exchangeProgram,
        });

        // const modifyComputeUnits = ComputeBudgetProgram.setComputeUnitLimit({
        //     units: 1000000
        // });
        let tx = new Transaction();
        //tx.add(modifyComputeUnits);
        //tx.recentBlockhash = (await connection.getLatestBlockhash('finalized')).blockhash;
        tx.feePayer = payer.publicKey;
        tx.add(ix);

        let sim_result = await connection.simulateTransaction(tx);
        console.log("logs : {}", sim_result.value.logs);

        // let tx = new Transaction();
        // tx.recentBlockhash = (await connection.getLatestBlockhash('finalized')).blockhash;
        // tx.feePayer = clientWallet.publicKey;
        // tx.add(ix);
        //
        // let committed = await sendAndConfirmTransaction(
        //     connection,
        //     tx,
        //     [clientWallet, exchangeWallet],
        //     {
        //         skipPreflight: true,
        //         preflightCommitment : "finalized"
        //     }
        // );
        // console.log("transaction: {}", committed)
    });
});
