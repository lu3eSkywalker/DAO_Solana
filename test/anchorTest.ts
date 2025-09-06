import * as anchor from "@coral-xyz/anchor";
import * as web3 from "@solana/web3.js";
import { CpiGuardLayout, createAssociatedTokenAccountInstruction, getAccount, getAssociatedTokenAddress, transfer } from "@solana/spl-token";
import { DaoContract } from "../target/deploy/DaoContract";
import { BN } from "bn.js";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";
import { assert } from "chai";

describe("Test", () => {
  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.DaoContract as anchor.Program<DaoContract>;

  // Generate a new keypair for the DAO account
  const daoAccountKeypair = web3.Keypair.generate();

  // Generate a new proposal keypair for the proposal account
  const proposalKeypair = web3.Keypair.generate();

  it("creates a dao", async () => {
    const user1PublicKey = new web3.PublicKey("5YLbUx2MGaHvSV1de5Kr1dVWPupbf63Mm5a9VhtvqoNt");
    const user2PublicKey = new web3.PublicKey("7eacdg5tZYPPqNdhi9PHvP5TUCEt9RjgUyoJL1a6L8JA");
    const user3PublicKey = new web3.PublicKey("8tbeZfMaQRfqYVCeaL5gnjn7nGMeKezYNe7c6tLwAK5X");
    const user4PublicKey = new web3.PublicKey("HVw1Z2KFYfKjdL2UThi5RGBvSUpsF4zdsPrucV8TggQm");

    
    const members = [
      program.provider.publicKey,
      user1PublicKey,
      user2PublicKey,
      user3PublicKey,
      user4PublicKey
    ];

    const LAMPORTS_PER_SOL = 1_000_000_000 * 0.2;

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

  it("create a dao proposal", async() => {

    const daoPubkey = daoAccountKeypair.publicKey;
    const title = "Token Minting";
    const description = "Should we mint more tokens?";
    
    const options = [
      { text: "Yes", vote_count: new anchor.BN(0) },
      { text: "No", vote_count: new anchor.BN(0) }
    ];

   const instructionData = Buffer.from([
      0x3b, 0x84, 0x18, 0xf6, 0x7a, 0x27, 0x08, 0xf3,
      0x00, 0xf2, 0x05, 0x2a, 0x01, 0x00, 0x00, 0x00
    ]);

    const programId = new web3.PublicKey("9xBdHWanyjR6U84K7p59A2757bXH7Zhjvh3NdbkMbuwb");

    const proposer = program.provider.publicKey;

    const txHash = await program.methods
    .createProposal(daoPubkey, title, description, programId, instructionData, options, proposer)
    .accounts({   
      proposal: proposalKeypair.publicKey,
      proposer: program.provider.publicKey,
      daoinfo: daoAccountKeypair.publicKey,
      systemProgram: web3.SystemProgram.programId,
    })
    .signers([proposalKeypair])
    .rpc()

    console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

    // Confirm Transaction
    await program.provider.connection.confirmTransaction(txHash);
    
    const proposalAccount = await program.account.proposal.fetch(proposalKeypair.publicKey);

    console.log("proposalAccount data: ", proposalAccount);

    assert.equal(proposalAccount.title, "Token Minting");
    assert.equal(proposalAccount.description, "Should we mint more tokens?");
    assert.equal(proposalAccount.proposer, program.provider.publicKey.toString());
    assert.equal(proposalAccount.executed, false);
  });

  it("user 1 casts a vote to the proposal", async() => {

    const user1PrivateKey = "";
    const privateKeySeed = bs58.decode(user1PrivateKey);

    const userKeyPair = web3.Keypair.fromSecretKey(privateKeySeed);

    const option_index = 1;

    const txHash = await program.methods
    .vote(option_index)
    .accounts({
      voter: userKeyPair.publicKey,
      daoinfo: daoAccountKeypair.publicKey,
      proposal: proposalKeypair.publicKey,
      systemProgram: web3.SystemProgram.programId
    })
    .signers([userKeyPair])
    .rpc()

    console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

    // Confirm Transaction
    await program.provider.connection.confirmTransaction(txHash);

    const proposalAccount = await program.account.proposal.fetch(proposalKeypair.publicKey);

    console.log("proposalAccount voter: ", proposalAccount.voters);

    assert.equal(proposalAccount.voters[0], "5YLbUx2MGaHvSV1de5Kr1dVWPupbf63Mm5a9VhtvqoNt");
  })
});