"use client";

import WalletButton from "@/components/ui/wallet-button";
import { useWallet } from "@solana/wallet-adapter-react";


export default function Home() {
  const { publicKey, connected } = useWallet();
  if (!connected) return <WalletButton />;


  return (
    <div className="flex min-h-screen flex-col items-center justify-between p-24">
      <p>Connected: {publicKey?.toBase58()}</p>
    </div>
  );
}
