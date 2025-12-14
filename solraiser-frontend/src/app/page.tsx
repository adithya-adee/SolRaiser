"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import Profile from "@/components/profile";
import WalletButton from "@/components/ui/wallet-button";
import { WalletInfo } from "@/lib/utils/wallet-info";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";

export default function Home() {
  const [mounted, setMounted] = useState(false);
  const { connected, publicKey } = WalletInfo();

  // Ensure component only renders wallet-dependent UI after hydration
  useEffect(() => {
    setMounted(true);
  }, []);

  const navItems = [
    {
      title: "Create Campaign",
      description: "Launch your fundraising campaign on the Solana blockchain",
      icon: "🚀",
      route: "/create",
      gradient: "from-blue-500 to-cyan-500",
      hoverGradient: "hover:from-blue-600 hover:to-cyan-600",
    },
    {
      title: "View Campaigns",
      description: "Discover and support amazing projects in our community",
      icon: "🌟",
      route: "/campaigns",
      gradient: "from-purple-500 to-pink-500",
      hoverGradient: "hover:from-purple-600 hover:to-pink-600",
    },
    {
      title: "Your Profile",
      description: "Manage your campaigns and track your contributions",
      icon: "👤",
      route: "/profile",
      gradient: "from-cyan-500 to-blue-500",
      hoverGradient: "hover:from-cyan-600 hover:to-blue-600",
    },
    {
      title: "Dashboard",
      description:
        "View analytics and insights about your fundraising activity",
      icon: "📊",
      route: "/dashboard",
      gradient: "from-indigo-500 to-purple-500",
      hoverGradient: "hover:from-indigo-600 hover:to-purple-600",
    },
  ];

  return (
    <div className="min-h-screen">
      {/* Glassmorphism Navigation */}
      <nav className="fixed top-0 left-0 right-0 z-50 backdrop-blur-xl bg-slate-900/40 border-b border-white/10 shadow-lg">
        <div className="max-w-7xl mx-auto px-6 py-4 flex items-center justify-between">
          {/* Logo and Branding */}
          <div className="flex items-center gap-3">
            <div className="relative w-10 h-10 rounded-lg bg-gradient-to-br from-blue-500 via-cyan-500 to-purple-500 p-0.5 animate-pulse">
              <div className="w-full h-full bg-slate-900 rounded-lg flex items-center justify-center text-2xl">
                ☀️
              </div>
            </div>
            <h1 className="text-2xl font-bold bg-gradient-to-r from-blue-400 via-cyan-400 to-purple-400 bg-clip-text text-transparent">
              SolRaiser
            </h1>
          </div>

          {/* Wallet Section */}
          {mounted ? (
            connected ? (
              <Profile publicKey={publicKey?.toBase58() || ""} />
            ) : (
              <WalletButton />
            )
          ) : (
            <div className="w-32 h-10 bg-white/5 rounded-lg animate-pulse" />
          )}
        </div>
      </nav>

      {/* Hero Section */}
      <main className="pt-24 pb-16 px-6">
        <div className="max-w-7xl mx-auto">
          {/* Hero Text */}
          <div className="text-center mb-16 mt-12">
            <h2 className="text-5xl md:text-6xl font-bold mb-6 bg-gradient-to-r from-white via-blue-100 to-cyan-100 bg-clip-text text-transparent">
              Empower Dreams with Blockchain
            </h2>
            <p className="text-xl text-slate-300 max-w-2xl mx-auto">
              Transparent, secure, and decentralized crowdfunding powered by
              Solana. Support projects you believe in, create campaigns that
              matter.
            </p>
          </div>

          {/* Cards Grid */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            {navItems.map((item, index) => (
              <Link key={index} href={item.route} className="group">
                <Card className="card-hover-effect h-full bg-slate-800/40 backdrop-blur-sm border-white/10 hover:border-white/20 overflow-hidden relative">
                  {/* Gradient overlay on hover */}
                  <div
                    className={`absolute inset-0 bg-gradient-to-br ${item.gradient} opacity-0 group-hover:opacity-10 transition-opacity duration-300`}
                  />

                  <CardHeader>
                    {/* Icon */}
                    <div className="mb-4">
                      <div
                        className={`w-14 h-14 rounded-xl bg-gradient-to-br ${item.gradient} flex items-center justify-center text-3xl transform group-hover:scale-110 transition-transform duration-300`}
                      >
                        {item.icon}
                      </div>
                    </div>

                    <CardTitle className="text-white text-xl mb-2">
                      {item.title}
                    </CardTitle>
                    <CardDescription className="text-slate-400">
                      {item.description}
                    </CardDescription>
                  </CardHeader>

                  <CardContent>
                    <Button
                      variant="outline"
                      className={`w-full bg-gradient-to-r ${item.gradient} ${item.hoverGradient} border-0 text-white font-semibold shadow-lg group-hover:shadow-xl transition-all`}
                    >
                      Explore
                      <span className="ml-2 transform group-hover:translate-x-1 transition-transform">
                        →
                      </span>
                    </Button>
                  </CardContent>
                </Card>
              </Link>
            ))}
          </div>

          {/* Stats Section */}
          <div className="mt-20 grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="bg-slate-800/40 backdrop-blur-sm border border-white/10 rounded-xl p-6 text-center">
              <div className="text-4xl font-bold bg-gradient-to-r from-blue-400 to-cyan-400 bg-clip-text text-transparent mb-2">
                1,234
              </div>
              <div className="text-slate-400">Active Campaigns</div>
            </div>
            <div className="bg-slate-800/40 backdrop-blur-sm border border-white/10 rounded-xl p-6 text-center">
              <div className="text-4xl font-bold bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent mb-2">
                $2.5M+
              </div>
              <div className="text-slate-400">Total Raised</div>
            </div>
            <div className="bg-slate-800/40 backdrop-blur-sm border border-white/10 rounded-xl p-6 text-center">
              <div className="text-4xl font-bold bg-gradient-to-r from-cyan-400 to-blue-400 bg-clip-text text-transparent mb-2">
                10K+
              </div>
              <div className="text-slate-400">Community Members</div>
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}
