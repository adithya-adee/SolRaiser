import { useAnchorWallet, useConnection } from "@solana/wallet-adapter-react";
import idl from "@/idl/solraiser.json";
import { AnchorProvider, Program, Wallet } from "@coral-xyz/anchor";
import { Keypair } from "@solana/web3.js";

export default function useAnchorProgram() {
  const { connection } = useConnection();
  const wallet = useAnchorWallet();

  // Create a dummy wallet when no wallet is connected (for read-only operations)
  const dummyKeypair = Keypair.generate();
  const dummyWallet: Wallet = {
    publicKey: dummyKeypair.publicKey,
    payer: dummyKeypair,
    signTransaction: async () => {
      throw new Error("Wallet not connected");
    },
    signAllTransactions: async () => {
      throw new Error("Wallet not connected");
    },
  };

  const provider = new AnchorProvider(
    connection,
    wallet ?? dummyWallet,
    { commitment: "confirmed" },
  );

  const program = new Program(idl as any, provider);

  return program;
}
