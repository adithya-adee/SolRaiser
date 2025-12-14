"use client";

import { useWallet } from "@solana/wallet-adapter-react";

export function WalletInfo() {
  const { publicKey, connected } = useWallet();

  if (!connected) {
    return {
      connected: false,
      publicKey: null,
    };
  }

  return {
    connected: true,
    publicKey,
  };
}
