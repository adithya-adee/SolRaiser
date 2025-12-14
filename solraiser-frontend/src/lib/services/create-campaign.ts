import { BN, Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";

interface CreateCampaignParams {
  program: Program;
  campaignId: number;
  goalAmount: number;
  deadline: number;
  metadataUrl: string;
  creatorPublicKey: PublicKey;
}

/**
 * Creates a new campaign on the Solana blockchain
 * @param params - Campaign creation parameters
 * @returns Transaction signature
 */
export async function createCampaign(params: CreateCampaignParams): Promise<string> {
  const { program, campaignId, goalAmount, deadline, metadataUrl, creatorPublicKey } = params;

  try {
    // Convert numbers to BN for Anchor
    const campaignIdBN = new BN(campaignId);
    const goalLamports = new BN(Math.floor(goalAmount * 1e9)); // Convert SOL to lamports
    const deadlineBN = new BN(deadline);

    // Derive the campaign PDA
    const [campaignPda, _campaignBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("campaign"),
        creatorPublicKey.toBuffer(),
        campaignIdBN.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    console.log("Creating campaign with PDA:", campaignPda.toString());

    // Call the program
    const txSignature = await program.methods
      .createCampaign(campaignIdBN, goalLamports, deadlineBN, metadataUrl)
      .accounts({
        campaignAccount: campaignPda,
        creator: creatorPublicKey,
      })
      .rpc();

    console.log("Campaign created successfully. Signature:", txSignature);
    return txSignature;
  } catch (error) {
    console.error("Error creating campaign:", error);
    
    // Extract meaningful error message
    if (error instanceof Error) {
      throw new Error(`Failed to create campaign: ${error.message}`);
    }
    throw error;
  }
}