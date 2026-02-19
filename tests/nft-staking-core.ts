import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { NftStakingCore } from "../target/types/nft_staking_core";
import { SystemProgram } from "@solana/web3.js";
import { MPL_CORE_PROGRAM_ID } from "@metaplex-foundation/mpl-core";
import { ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { assert } from "chai";

const MILLISECONDS_PER_DAY = 86400000;
const MILLISECONDS_PER_HOUR = 3600000;
const POINTS_PER_STAKED_NFT_PER_DAY = 10_000_000;
const POINTS_PER_BURNED_NFT = 1_000_000_000;
const FREEZE_PERIOD_IN_DAYS = 7;
const TIME_TRAVEL_IN_DAYS = 8;
const OPEN_HOUR = 9 * MILLISECONDS_PER_HOUR;

describe("nft-staking-core", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.nftStakingCore as Program<NftStakingCore>;

  // Generate a keypair for the new owner
  const newOwnerKeypair = anchor.web3.Keypair.generate();

  // Generate a keypair for the collection
  const collectionKeypair = anchor.web3.Keypair.generate();

  // Find the update authority for the collection (PDA)
  const updateAuthority = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("update_authority"), collectionKeypair.publicKey.toBuffer()],
    program.programId
  )[0];

  // Generate a keypair for the nft asset
  const nftKeypair = anchor.web3.Keypair.generate();

  // Find the config account (PDA)
  const config = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("config"), collectionKeypair.publicKey.toBuffer()],
    program.programId
  )[0];

  // Find the rewards mint account (PDA)
  const rewardsMint = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("rewards"), config.toBuffer()],
    program.programId
  )[0];

  // Find the oracle account (PDA)
  const oracle = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("oracle")],
    program.programId
  )[0];

  // Find the rewardsvault account (PDA)
  const vault = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), oracle.toBuffer()],
    program.programId
  )[0];

  /**
   * Helper function to advance time with surfnet_timeTravel RPC method
   * @param params - Time travel params (absoluteEpoch, absoluteSlot, or absoluteTimestamp)
   */
  async function advanceTime(params: { absoluteEpoch?: number; absoluteSlot?: number; absoluteTimestamp?: number }): Promise<void> {
    const rpcResponse = await fetch(provider.connection.rpcEndpoint, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method: "surfnet_timeTravel",
        params: [params],
      }),
    });

    const result = await rpcResponse.json() as { error?: any; result?: any };
    if (result.error) {
      throw new Error(`Time travel failed: ${JSON.stringify(result.error)}`);
    }
    
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }

  it("Initialize oracle account", async () => {
    const tx = await program.methods.initOracle()
    .accountsPartial({
      user: provider.wallet.publicKey,
      oracle,
      vault,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
    console.log("\nYour transaction signature", tx);
    console.log("Oracle address", oracle.toBase58());
    console.log("Vault address", vault.toBase58());
  });

  it("Fund the vault account", async () => {
    const tx = await provider.connection.requestAirdrop(vault, 1_000_000_000);
  });

  xit("Create a collection", async () => {
    const collectionName = "Test Collection";
    const collectionUri = "https://example.com/collection";
    const tx = await program.methods.createCollection(collectionName, collectionUri)
    .accountsPartial({
      payer: provider.wallet.publicKey,
      collection: collectionKeypair.publicKey,
      updateAuthority,
      systemProgram: SystemProgram.programId,
      mplCoreProgram: MPL_CORE_PROGRAM_ID,
    })
    .signers([collectionKeypair])
    .rpc();
    console.log("\nYour transaction signature", tx);
    console.log("Collection address", collectionKeypair.publicKey.toBase58());
  });

  it("Create a collection with oracle plugin", async () => {
    const collectionName = "Oracle Collection";
    const collectionUri = "https://example.com/my-collection";
    const tx = await program.methods.createCollectionWithOracle(collectionName, collectionUri, oracle)
    .accountsPartial({
      payer: provider.wallet.publicKey,
      collection: collectionKeypair.publicKey,
      updateAuthority,
      systemProgram: SystemProgram.programId,
      mplCoreProgram: MPL_CORE_PROGRAM_ID,
    })
    .signers([collectionKeypair])
    .rpc();
    console.log("\nYour transaction signature", tx);
    console.log("Collection address", collectionKeypair.publicKey.toBase58());
  });

  it("Mint an NFT", async () => {
    const nftName = "Test NFT";
    const nftUri = "https://example.com/nft";
    const tx = await program.methods.mintNft(nftName, nftUri)
    .accountsPartial({
      user: provider.wallet.publicKey,
      nft: nftKeypair.publicKey,
      collection: collectionKeypair.publicKey,
      updateAuthority,
      systemProgram: SystemProgram.programId,
      mplCoreProgram: MPL_CORE_PROGRAM_ID,
    })
    .signers([nftKeypair])
    .rpc();
    console.log("\nYour transaction signature", tx);
    console.log("NFT address", nftKeypair.publicKey.toBase58());
  });

  it("Try to transfer NFT outside trading allowed hours", async () => {
    try {
      await program.methods.transferNft()
      .accountsPartial({
        owner: provider.wallet.publicKey,
        newOwner: newOwnerKeypair.publicKey,
        oracle,
        nft: nftKeypair.publicKey,
        collection: collectionKeypair.publicKey,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      .rpc();
      // If we reach here, the test should fail because we expected an error
      throw new Error("\nExpected transaction to fail, but it succeeded");
    } catch (error) {
      console.log("\nTransaction failed as expected");
    }
    
    
  });

  it("Time travel to update oracle", async () => {
    // Advance time to trading allowed hours
    const currentTimestamp = Date.now();
    const timeSinceMidnight = currentTimestamp % MILLISECONDS_PER_DAY;
    const timeToOpen = OPEN_HOUR - timeSinceMidnight;
    if (timeToOpen > 0) {
      await advanceTime({ absoluteTimestamp: currentTimestamp + timeToOpen });
      console.log("\nTime traveled to update oracle", timeToOpen);
    } else {
      console.log("\nTime already in trading allowed hours");
    }
  });

  it("Update oracle account", async () => {
    const tx = await program.methods.updateOracle()
    .accountsPartial({
      signer: provider.wallet.publicKey,
      oracle,
      vault,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
    console.log("\nYour transaction signature", tx);
  });

  it("Transfer NFT within trading allowed hours", async () => {
    const tx = await program.methods.transferNft()
    .accountsPartial({
      owner: provider.wallet.publicKey,
      newOwner: newOwnerKeypair.publicKey,
      oracle,
      nft: nftKeypair.publicKey,
      collection: collectionKeypair.publicKey,
      systemProgram: SystemProgram.programId,
      mplCoreProgram: MPL_CORE_PROGRAM_ID,
    })
    .rpc();
    console.log("\nYour transaction signature", tx);
    
  });

  xit("Initialize stake config", async () => {
    const tx = await program.methods.initializeConfig(POINTS_PER_STAKED_NFT_PER_DAY, POINTS_PER_BURNED_NFT, FREEZE_PERIOD_IN_DAYS)
    .accountsPartial({
      admin: provider.wallet.publicKey,
      collection: collectionKeypair.publicKey,
      updateAuthority,
      config,
      rewardsMint,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .rpc();
    console.log("\nYour transaction signature", tx);
    console.log("Config address", config.toBase58());
    console.log("Points per staked NFT per day", POINTS_PER_STAKED_NFT_PER_DAY);
    console.log("Points per burned NFT", POINTS_PER_BURNED_NFT);
    console.log("Freeze period in days", FREEZE_PERIOD_IN_DAYS);
    console.log("Rewards mint address", rewardsMint.toBase58());
  });

  xit("Stake an NFT", async () => {
    const tx = await program.methods.stake()
    .accountsPartial({
      user: provider.wallet.publicKey,
      updateAuthority,
      config,
      nft: nftKeypair.publicKey,
      collection: collectionKeypair.publicKey,
      systemProgram: SystemProgram.programId,
      mplCoreProgram: MPL_CORE_PROGRAM_ID,
    })
    .rpc();
    console.log("\nYour transaction signature", tx);
  });

  xit("Attempt to unstake before freezing period", async () => {
    try {
      await program.methods.unstake()
        .accountsPartial({
          user: provider.wallet.publicKey,
          updateAuthority,
          config,
          nft: nftKeypair.publicKey,
          collection: collectionKeypair.publicKey,
          mplCoreProgram: MPL_CORE_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      // If we reach here, the test should fail because we expected an error
      throw new Error("\nExpected transaction to fail, but it succeeded");
    } catch (error) {
      assert.include(error.toString(), "NFT freeze period not elapsed");
      console.log("\nTransaction failed as expected: NFT freeze period not elapsed");
    }
  });

  xit("Time travel to the future to allow unstaking", async () => {
    // Advance time in milliseconds
    const currentTimestamp = Date.now();
    await advanceTime({ absoluteTimestamp: currentTimestamp + TIME_TRAVEL_IN_DAYS * MILLISECONDS_PER_DAY });
    console.log("\nTime traveled in days", TIME_TRAVEL_IN_DAYS)
  });

  xit("Claim rewards", async () => {
    // Get the user rewards ATA account
    const userRewardsAta = getAssociatedTokenAddressSync(rewardsMint, provider.wallet.publicKey, false, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID);
    const tx = await program.methods.claimRewards()
    .accountsPartial({
      user: provider.wallet.publicKey,
      config,
      rewardsMint,
      userRewardsAta,
      updateAuthority,
      nft: nftKeypair.publicKey,
      collection: collectionKeypair.publicKey,
      systemProgram: SystemProgram.programId,
      mplCoreProgram: MPL_CORE_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    })
    .rpc();
    console.log("\nYour transaction signature", tx);
    console.log("User rewards balance", (await provider.connection.getTokenAccountBalance(userRewardsAta)).value.uiAmount);
  });

  xit("Unstake an NFT", async () => {
    const tx = await program.methods.unstake()
    .accountsPartial({
      user: provider.wallet.publicKey,
      updateAuthority,
      config,
      nft: nftKeypair.publicKey,
      collection: collectionKeypair.publicKey,
      mplCoreProgram: MPL_CORE_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
    console.log("\nYour transaction signature", tx);
  });

  xit("Burn a staked NFT", async () => {
    // Get the user rewards ATA account
    const userRewardsAta = getAssociatedTokenAddressSync(rewardsMint, provider.wallet.publicKey, false, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID);
    const tx = await program.methods.burnStakedNft()
    .accountsPartial({
      user: provider.wallet.publicKey,
      config,
      rewardsMint,
      userRewardsAta,
      updateAuthority,
      nft: nftKeypair.publicKey,
      collection: collectionKeypair.publicKey,
      systemProgram: SystemProgram.programId,
      mplCoreProgram: MPL_CORE_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    })
    .rpc();
    console.log("\nYour transaction signature", tx);
    console.log("User rewards balance", (await provider.connection.getTokenAccountBalance(userRewardsAta)).value.uiAmount);
  });


});
