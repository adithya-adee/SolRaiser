"use client";

import { useWallet } from "@solana/wallet-adapter-react";
import { useWalletModal } from "@solana/wallet-adapter-react-ui";
import { CheckIcon, ChevronDown, CopyIcon, LogOut, Wallet } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";
import { Button } from "./ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "./ui/dropdown-menu";

export default function Profile({ publicKey }: { publicKey: string }) {
  const [copied, setCopied] = useState(false);
  const { disconnect } = useWallet();
  const { setVisible } = useWalletModal();

  const copyToClipboard = async () => {
    try {
      await navigator.clipboard.writeText(publicKey);
      setCopied(true);
      toast.success("Address copied to clipboard!", {
        description: shortenAddress(publicKey),
      });
      setTimeout(() => {
        setCopied(false);
      }, 2000);
    } catch (error) {
      console.error("Failed to copy to clipboard:", error);
      toast.error("Failed to copy address");
    }
  };

  const handleChangeWallet = () => {
    setVisible(true);
    toast.info("Wallet selector opened");
  };

  const handleDisconnect = () => {
    disconnect();
    toast.success("Wallet disconnected successfully");
  };

  const shortenAddress = (address: string) => {
    return `${address.slice(0, 4)}...${address.slice(-4)}`;
  };

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="ghost"
          className="group relative flex items-center gap-3 bg-[rgb(var(--solraiser-violet-600))]/20 backdrop-blur-md border border-[rgb(var(--solraiser-violet-500))]/30 rounded-2xl px-4 py-2.5 hover:bg-[rgb(var(--solraiser-violet-600))]/30 hover:border-[rgb(var(--solraiser-violet-400))]/50 hover:shadow-lg hover:shadow-[rgb(var(--solraiser-violet-500))]/20 transition-all duration-300"
        >
          {/* Animated gradient border effect */}
          <div className="absolute inset-0 rounded-2xl bg-gradient-to-r from-[rgb(var(--solraiser-violet-500))] via-[rgb(var(--solraiser-fuchsia-500))] to-[rgb(var(--solraiser-violet-500))] opacity-0 group-hover:opacity-20 blur transition-opacity duration-300" />

          {/* Public Key Display */}
          <div className="flex items-center gap-3">
            <span className="text-sm font-medium text-[rgb(var(--solraiser-violet-300))]/80 group-hover:text-[rgb(var(--solraiser-violet-300))] transition-colors">
              Connected
            </span>
            <p className="text-white text-sm font-mono font-semibold tracking-wide">
              {shortenAddress(publicKey)}
            </p>
          </div>

          {/* Dropdown Indicator */}
          <ChevronDown className="w-4 h-4 text-[rgb(var(--solraiser-violet-300))] group-hover:text-white transition-colors" />
        </Button>
      </DropdownMenuTrigger>

      <DropdownMenuContent
        className="w-64 bg-[rgb(var(--solraiser-slate-800))]/95 backdrop-blur-xl border-white/10 shadow-2xl"
        align="end"
      >
        <DropdownMenuLabel className="text-slate-300">
          <div className="flex flex-col gap-1">
            <span className="text-xs text-slate-400">Connected Wallet</span>
            <span className="text-sm font-mono text-white break-all">
              {publicKey}
            </span>
          </div>
        </DropdownMenuLabel>

        <DropdownMenuSeparator className="bg-white/10" />

        {/* Copy Address */}
        <DropdownMenuItem
          onClick={copyToClipboard}
          className="flex items-center gap-3 cursor-pointer hover:bg-white/10 focus:bg-white/10 text-slate-300 hover:text-white transition-colors"
        >
          {copied ? (
            <>
              <CheckIcon className="w-4 h-4 text-[rgb(var(--solraiser-green-400))]" />
              <span className="text-[rgb(var(--solraiser-green-400))]">
                Copied!
              </span>
            </>
          ) : (
            <>
              <CopyIcon className="w-4 h-4" />
              <span>Copy Address</span>
            </>
          )}
        </DropdownMenuItem>

        {/* Change Wallet */}
        <DropdownMenuItem
          onClick={handleChangeWallet}
          className="flex items-center gap-3 cursor-pointer hover:bg-white/10 focus:bg-white/10 text-slate-300 hover:text-white transition-colors"
        >
          <Wallet className="w-4 h-4" />
          <span>Change Wallet</span>
        </DropdownMenuItem>

        <DropdownMenuSeparator className="bg-white/10" />

        {/* Disconnect */}
        <DropdownMenuItem
          onClick={handleDisconnect}
          className="flex items-center gap-3 cursor-pointer hover:bg-[rgb(var(--solraiser-red-500))]/20 focus:bg-[rgb(var(--solraiser-red-500))]/20 text-slate-300 hover:text-[rgb(var(--solraiser-red-500))] transition-colors"
        >
          <LogOut className="w-4 h-4" />
          <span>Disconnect</span>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
