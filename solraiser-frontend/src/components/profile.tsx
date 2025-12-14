"use client";

import { useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { useWalletModal } from "@solana/wallet-adapter-react-ui";
import {
    CheckIcon,
    CopyIcon,
    User,
    LogOut,
    Wallet,
    ChevronDown,
} from "lucide-react";
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
            setTimeout(() => {
                setCopied(false);
            }, 2000);
        } catch (error) {
            console.error("Failed to copy to clipboard:", error);
        }
    };

    const handleChangeWallet = () => {
        setVisible(true);
    };

    const handleDisconnect = () => {
        disconnect();
    };

    const shortenAddress = (address: string) => {
        return `${address.slice(0, 4)}...${address.slice(-4)}`;
    };

    return (
        <DropdownMenu>
            <DropdownMenuTrigger asChild>
                <Button
                    variant="ghost"
                    className="group relative flex items-center gap-3 bg-gradient-to-br from-violet-600/20 to-fuchsia-600/20 backdrop-blur-md border border-violet-500/30 rounded-2xl px-4 py-2.5 hover:from-violet-600/30 hover:to-fuchsia-600/30 hover:border-violet-400/50 hover:shadow-lg hover:shadow-violet-500/20 transition-all duration-300"
                >
                    {/* Animated gradient border effect */}
                    <div className="absolute inset-0 rounded-2xl bg-gradient-to-r from-violet-500 via-fuchsia-500 to-violet-500 opacity-0 group-hover:opacity-20 blur transition-opacity duration-300" />

                    {/* User Icon */}
                    <div className="relative w-9 h-9 rounded-full bg-gradient-to-br from-violet-500 via-fuchsia-500 to-pink-500 flex items-center justify-center shadow-lg ring-2 ring-violet-400/30 group-hover:ring-violet-400/50 transition-all duration-300">
                        <User className="w-5 h-5 text-white" />
                    </div>

                    {/* Public Key Display */}
                    <div className="flex flex-col items-start">
                        <span className="text-xs font-medium text-violet-300/80 group-hover:text-violet-200 transition-colors">Connected</span>
                        <p className="text-white text-sm font-mono font-semibold tracking-wide">
                            {shortenAddress(publicKey)}
                        </p>
                    </div>

                    {/* Dropdown Indicator */}
                    <ChevronDown className="w-4 h-4 text-violet-300 group-hover:text-white transition-colors" />
                </Button>
            </DropdownMenuTrigger>

            <DropdownMenuContent
                className="w-64 bg-slate-800/95 backdrop-blur-xl border-white/10 shadow-2xl"
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
                            <CheckIcon className="w-4 h-4 text-green-400" />
                            <span className="text-green-400">Copied!</span>
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
                    className="flex items-center gap-3 cursor-pointer hover:bg-red-500/20 focus:bg-red-500/20 text-slate-300 hover:text-red-400 transition-colors"
                >
                    <LogOut className="w-4 h-4" />
                    <span>Disconnect</span>
                </DropdownMenuItem>
            </DropdownMenuContent>
        </DropdownMenu>
    );
}
