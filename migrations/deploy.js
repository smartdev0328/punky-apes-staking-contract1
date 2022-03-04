// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

const anchor = require("@project-serum/anchor");
const { AccountLayout, TOKEN_PROGRAM_ID, Token } = require('@solana/spl-token');
const IDL = require('../target/idl/punky_staking.json');
const { SystemProgram, Keypair, PublicKey } = anchor.web3;
const token_mint = 'GnBw4qZs3maF2d5ziQmGzquQFnGV33NUcEujTQ3CbzP3';
const PROGRAM_ID = '3n7nc4FUKJRxhovqwxV9XfYahRcjbxwnaFp1ZSXWqAZ1';

module.exports = async function (provider) {
  // Configure client to use the provider.
  anchor.setProvider(provider);
  const program = new anchor.Program(IDL, new PublicKey(PROGRAM_ID), provider);
  let [vaultPDA, _nonce] = await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from('NFT STAKING VAULT')],
    program.programId
  );

  const aTokenAccount = new Keypair();
  const aTokenAccountRent = await provider.connection.getMinimumBalanceForRentExemption(
    AccountLayout.span
  )

  console.log('vaultPda', vaultPDA.toString(), 'nonce', _nonce);
  console.log('tokenAccount', aTokenAccount.publicKey.toString());

  const tx = await program.rpc.createVault(
     _nonce, {
      accounts: {
        vault: vaultPDA,
        admin: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId
      },
      signers: [aTokenAccount],
      instructions: [
        SystemProgram.createAccount({
          fromPubkey: provider.wallet.publicKey,
          newAccountPubkey: aTokenAccount.publicKey,
          lamports: aTokenAccountRent,
          space: AccountLayout.span,
          programId: TOKEN_PROGRAM_ID
        }),
        Token.createInitAccountInstruction(
          TOKEN_PROGRAM_ID,
          new PublicKey(token_mint),
          aTokenAccount.publicKey,
          vaultPDA
        )
      ]
    } 
  );
  console.log('migration tx', tx);

  let [poolData, nonce] = await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from('NFT STAKING DATA')],
    program.programId
  );
  console.log('poolData', poolData);
  const tx_data = await program.rpc.createDataAccount(nonce, {
    accounts: {
      data: poolData,
      admin: provider.wallet.publicKey,
      systemProgram: SystemProgram.programId
    }
  })

  console.log('tx data', tx_data);
}
