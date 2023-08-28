# solana_simple_exchange
Simple token exchange for Solana network

Solana Program is the analogue of smart contract in the Solana Network.
SOL is the native Solana coin.
The purpose of this Solana Program is to exchange:

- native SOL  ---> Any Solana Token
- Any Solana Token ---> native SOL
- Any Solana Token ---> Any Solana Token

programId = G8vMwzB6DXD7E3zz4Xwm9x2JcZsiR7zhiKKimiMeFarS  (devnet)
devnet explorer - https://explorer.solana.com/?cluster=devnet

To exchange tokens you need to pass following steps (if you aren't already Solana dev or user):
- install Solana CLI tools https://docs.solana.com/cli/install-solana-cli-tools
- Generate a File System Wallet Keypair (the simplest variant) https://docs.solana.com/wallet-guide/file-system-wallet
- solana config set --url https://api.devnet.solana.com
- solana airdrop 1 <RECIPIENT_ACCOUNT_ADDRESS> --url https://api.devnet.solana.com 
- The simplest way to perform exchange is to use /tests/test.ts
  Create ATA(Associated Token Account https://spl.solana.com/token ) with Solana CLI 
  Change const clientWrappedSolAccount / const clientUsdcAssociatedTokenAccount / const clientCustomAssociatedTokenAccount
  to values created by spl-token
  run tests, watch results in https://explorer.solana.com/?cluster=devnet
