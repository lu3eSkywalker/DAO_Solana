import * as anchor from "@coral-xyz/anchor";
import * as web3 from "@solana/web3.js";
import { CpiGuardLayout, createAssociatedTokenAccountInstruction, getAccount, getAssociatedTokenAddress, transfer } from "@solana/spl-token";
import { DaoContract } from "../target/deploy/DaoContract";
import { BN } from "bn.js";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";

describe("Test", () => {
  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.DaoContract as anchor.Program<DaoContract>;

  // Generate a new keypair for the DAO account
  const daoAccountKeypair = web3.Keypair.generate();

  // Generate a new transasction keypair for the multisig account
  const transactionKeypair = web3.Keypair.generate();

  it("creates a multisig", async () => {

    const user1PublicKey = new web3.PublicKey("HVw1Z2KFYfKjdL2UThi5RGBvSUpsF4zdsPrucV8TggQm");
    const user2PublicKey = new web3.PublicKey("7eacdg5tZYPPqNdhi9PHvP5TUCEt9RjgUyoJL1a6L8JA");
    const user3PublicKey = new web3.PublicKey("8tbeZfMaQRfqYVCeaL5gnjn7nGMeKezYNe7c6tLwAK5X");
    const user4PublicKey = new web3.PublicKey("5YLbUx2MGaHvSV1de5Kr1dVWPupbf63Mm5a9VhtvqoNt");

    const members = [
      program.provider.publicKey,
      user1PublicKey,
      user2PublicKey,
      user3PublicKey,
      user4PublicKey
    ];

    const LAMPORTS_PER_SOL = 1_000_000_000;

    const transferTx = new web3.Transaction().add(
      web3.SystemProgram.transfer({
        fromPubkey: program.provider.publicKey,
        toPubkey: daoAccountKeypair.publicKey,
        lamports: LAMPORTS_PER_SOL,
      })
    );

    await program.provider.sendAndConfirm(transferTx);

    // Initialize the DAO Account
    const txHash = await program.methods
    .createDao(members)
    .accounts({
      daoinfo: daoAccountKeypair.publicKey,
      payer: program.provider.publicKey,
      systemProgram: web3.SystemProgram.programId,
    })
    .signers([daoAccountKeypair])
    .rpc();

    console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

    // Confirm Transaction
    await program.provider.connection.confirmTransaction(txHash);

    const daoAccount = await program.provider.connection.getAccountInfo(daoAccountKeypair.publicKey);
    console.log("Multisig account created with data: ", daoAccount);
  });
});