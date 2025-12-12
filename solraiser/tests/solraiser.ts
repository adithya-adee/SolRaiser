import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Solraiser } from "../target/types/solraiser";
import { Keypair, PublicKey, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { assert } from "chai";

describe("solraiser", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.solraiser as Program<Solraiser>;

  // Test accounts
  let creator: Keypair;
  let donor: Keypair;
  let otherUser: Keypair;

  // Constants
  const MIN_SOL_BALANCE = 5 * LAMPORTS_PER_SOL;

  // Helper to process transactions
  async function confirmTransaction(tx: string) {
    const latestBlockHash = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: tx,
    });
  }

  // Reliable airdrop helper
  async function airdrop(user: PublicKey, amount: number) {
    try {
      const signature = await provider.connection.requestAirdrop(
        user,
        amount
      );
      await confirmTransaction(signature);
    } catch (e) {
      console.log(`Airdrop failed: ${e}, check if local validator is running or user has funds`);
      // Fallback: This might fail if the user doesn't have funds on devnet, 
      // but strictly for local-validator this should work.
    }
  }

  before(async () => {
    creator = Keypair.generate();
    donor = Keypair.generate();
    otherUser = Keypair.generate();

    // Airdrop to all actors
    await Promise.all([
      airdrop(creator.publicKey, MIN_SOL_BALANCE),
      airdrop(donor.publicKey, MIN_SOL_BALANCE),
      airdrop(otherUser.publicKey, MIN_SOL_BALANCE),
    ]);
  });

  // Unique campaign ID generator to ensure tests don't collide on repeated runs
  const generateCampaignId = () => new BN(Math.floor(Math.random() * 1_000_000_000));

  // Helper to derive PDA
  function getCampaignAddress(creatorKey: PublicKey, campaignsId: BN) {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from("campaign"),
        creatorKey.toBuffer(),
        campaignsId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    )[0];
  }

  it("Creates a campaign successfully", async () => {
    const campaignId = generateCampaignId();
    const goalAmount = new BN(1 * LAMPORTS_PER_SOL);
    const deadline = new BN(Math.floor(Date.now() / 1000) + 60); // 1 minute from now
    const metadataUrl = "https://example.com/project";

    const campaignPda = getCampaignAddress(creator.publicKey, campaignId);

    await program.methods
      .createCampaign(campaignId, goalAmount, deadline, metadataUrl)
      .accountsPartial({
        creator: creator.publicKey,
        campaignAccount: campaignPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([creator])
      .rpc();

    const campaignAccount = await program.account.campaign.fetch(campaignPda);
    assert.ok(campaignAccount.creatorPubkey.equals(creator.publicKey));
    assert.ok(campaignAccount.campaignId.eq(campaignId));
    assert.ok(campaignAccount.goalAmount.eq(goalAmount));
    assert.ok(campaignAccount.amountRaised.eq(new BN(0)));
    assert.ok(campaignAccount.deadline.eq(deadline));
    assert.strictEqual(campaignAccount.metadataUrl, metadataUrl);
  });

  it("Donates to a campaign successfully", async () => {
    const campaignId = generateCampaignId();
    const goalAmount = new BN(5 * LAMPORTS_PER_SOL);
    const deadline = new BN(Math.floor(Date.now() / 1000) + 60);
    const metadataUrl = "https://example.com/donate";
    
    // 1. Create
    const campaignPda = getCampaignAddress(creator.publicKey, campaignId);
    await program.methods
      .createCampaign(campaignId, goalAmount, deadline, metadataUrl)
      .accountsPartial({
        creator: creator.publicKey,
        campaignAccount: campaignPda,
      })
      .signers([creator])
      .rpc();

    // 2. Donate
    const donationAmount = new BN(1 * LAMPORTS_PER_SOL);
    await program.methods
      .donate(donationAmount)
      .accountsPartial({
        campaignAccount: campaignPda,
        donor: donor.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([donor])
      .rpc();

    // Verify
    const campaignAccount = await program.account.campaign.fetch(campaignPda);
    assert.ok(campaignAccount.amountRaised.eq(donationAmount));
    
    // Check balance movement is roughly correct involves checking lamports before/after
    // but relying on program state is usually sufficient for checking logic correctness.
  });

  it("Fails to withdraw if goal not met", async () => {
    const campaignId = generateCampaignId();
    const goalAmount = new BN(10 * LAMPORTS_PER_SOL);
    // Deadline in past for withdrawal check? 
    // Wait, withdrawal requires (goal reached) AND (deadline passed).
    // If goal NOT reached, it should fail with GoalNotReached.
    
    // Set deadline in the past so "CampaignStillActive" isn't the error derived first
    const deadline = new BN(Math.floor(Date.now() / 1000) - 100); 

    const campaignPda = getCampaignAddress(creator.publicKey, campaignId);
    
    // Create with past deadline (program requires deadline > Clock::get() on create... 
    // wait, create_campaign checks deadline > now. So we must create with future deadline, 
    // then wait? Or we can't test "deadline passed" easily without wait or hack.)
    
    // Let's create with short deadline from now.
    const validDeadline = new BN(Math.floor(Date.now() / 1000) + 2); // 2 seconds
    
    await program.methods
      .createCampaign(campaignId, goalAmount, validDeadline, "fail_withdraw")
      .accountsPartial({
        creator: creator.publicKey,
        campaignAccount: campaignPda,
      })
      .signers([creator])
      .rpc();

    // Donate small amount (not reaching goal)
    await program.methods.donate(new BN(1 * LAMPORTS_PER_SOL))
      .accountsPartial({
        campaignAccount: campaignPda,
        donor: donor.publicKey,
      })
      .signers([donor])
      .rpc();

    // Wait for deadline to pass
    await new Promise(r => setTimeout(r, 3000));

    try {
      await program.methods.withdraw()
        .accountsPartial({
          campaignAccount: campaignPda,
          creator: creator.publicKey,
        })
        .signers([creator])
        .rpc();
      assert.fail("Should have failed with GoalNotReached");
    } catch (e: any) {
      assert.ok(JSON.stringify(e).includes("GoalNotReached") || e.error?.errorCode?.code === "GoalNotReached");
    }
  });

  it("Allows withdrawal if goal reached and deadline passed", async () => {
    const campaignId = generateCampaignId();
    const goalAmount = new BN(1 * LAMPORTS_PER_SOL); // Small goal
    const deadline = new BN(Math.floor(Date.now() / 1000) + 2); // 2 seconds

    const campaignPda = getCampaignAddress(creator.publicKey, campaignId);

    // 1. Create
    await program.methods.createCampaign(campaignId, goalAmount, deadline, "withdraw_ok")
      .accountsPartial({
        creator: creator.publicKey,
        campaignAccount: campaignPda,
      })
      .signers([creator])
      .rpc();

    // 2. Donate to goal
    await program.methods.donate(goalAmount)
      .accountsPartial({
        campaignAccount: campaignPda,
        donor: donor.publicKey,
      })
      .signers([donor])
      .rpc();

    // 3. Wait for deadline
    console.log("Waiting for deadline to pass...");
    await new Promise(r => setTimeout(r, 6000)); // Wait 6 seconds to be sure

    // Monitor balance
    const creatorBefore = await provider.connection.getBalance(creator.publicKey);
    
    // 4. Withdraw
    await program.methods.withdraw()
      .accountsPartial({
        campaignAccount: campaignPda,
        creator: creator.publicKey,
      })
      .signers([creator])
      .rpc();

    const creatorAfter = await provider.connection.getBalance(creator.publicKey);
    const campaignAccount = await program.account.campaign.fetch(campaignPda);

    // Goal amount should be transferred out, minus tx fees.
    assert.ok(campaignAccount.amountRaised.eq(new BN(0))); // Since we donated exactly goal amount
    assert.ok(creatorAfter > creatorBefore); 
  });
});
