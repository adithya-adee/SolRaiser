"use client";

import { useWallet } from "@solana/wallet-adapter-react";
import { CalendarIcon, Link2Icon, RocketIcon, Target } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import useAnchorProgram from "@/lib/utils/use-anchor-program";
import { createCampaign } from "@/lib/services/create-campaign";

export default function CreateCampaignPage() {
  const { publicKey, connected } = useWallet();
  const program = useAnchorProgram();
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Form state
  const [campaignId, setCampaignId] = useState("");
  const [goalAmount, setGoalAmount] = useState("");
  const [deadline, setDeadline] = useState("");
  const [metadataUrl, setMetadataUrl] = useState("");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!connected || !publicKey) {
      toast.error("Please connect your wallet first");
      return;
    }

    // Validation
    if (!campaignId || !goalAmount || !deadline || !metadataUrl) {
      toast.error("All fields are required");
      return;
    }

    const campaignIdNum = parseInt(campaignId);
    if (isNaN(campaignIdNum) || campaignIdNum < 0) {
      toast.error("Campaign ID must be a valid positive number");
      return;
    }

    const goalAmountNum = parseFloat(goalAmount);
    if (isNaN(goalAmountNum) || goalAmountNum <= 0) {
      toast.error("Goal amount must be greater than 0");
      return;
    }

    const deadlineDate = new Date(deadline);
    const deadlineTimestamp = Math.floor(deadlineDate.getTime() / 1000);
    const currentTimestamp = Math.floor(Date.now() / 1000);

    if (deadlineTimestamp <= currentTimestamp) {
      toast.error("Deadline must be in the future");
      return;
    }

    setIsSubmitting(true);

    try {
      // Create the campaign on Solana
      const txSignature = await createCampaign({
        program,
        campaignId: campaignIdNum,
        goalAmount: goalAmountNum,
        deadline: deadlineTimestamp,
        metadataUrl,
        creatorPublicKey: publicKey,
      });

      toast.success("Campaign created successfully!", {
        description: `Campaign ID: ${campaignId}`,
        action: {
          label: "View Transaction",
          onClick: () => {
            window.open(
              `https://explorer.solana.com/tx/${txSignature}?cluster=devnet`,
              "_blank"
            );
          },
        },
      });

      // Reset form
      setCampaignId("");
      setGoalAmount("");
      setDeadline("");
      setMetadataUrl("");
    } catch (error) {
      console.error("Failed to create campaign:", error);
      toast.error("Failed to create campaign", {
        description: error instanceof Error ? error.message : "Unknown error",
      });
    } finally {
      setIsSubmitting(false);
    }
  };

  if (!connected) {
    return (
      <div className="min-h-screen flex items-center justify-center p-6">
        <Card className="max-w-md w-full bg-[rgb(var(--solraiser-slate-800))]/95 backdrop-blur-xl border-white/10 shadow-2xl">
          <CardHeader>
            <CardTitle className="text-2xl text-white">
              Connect Your Wallet
            </CardTitle>
            <CardDescription className="text-slate-400">
              Please connect your wallet to create a campaign
            </CardDescription>
          </CardHeader>
        </Card>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-[rgb(var(--solraiser-slate-900))] via-[rgb(var(--solraiser-violet-950))] to-[rgb(var(--solraiser-slate-900))] p-6">
      <div className="max-w-2xl mx-auto py-12">
        {/* Header */}
        <div className="mb-8 text-center">
          <h1 className="text-4xl font-bold text-white mb-2 bg-gradient-to-r from-[rgb(var(--solraiser-violet-400))] via-[rgb(var(--solraiser-fuchsia-400))] to-[rgb(var(--solraiser-violet-400))] bg-clip-text text-transparent">
            Create Campaign
          </h1>
          <p className="text-slate-400 text-lg">
            Launch your fundraising campaign on Solana
          </p>
        </div>

        {/* Form Card */}
        <Card className="bg-[rgb(var(--solraiser-slate-800))]/95 backdrop-blur-xl border-white/10 shadow-2xl">
          <CardHeader>
            <CardTitle className="text-2xl text-white flex items-center gap-2">
              <RocketIcon className="w-6 h-6 text-[rgb(var(--solraiser-violet-400))]" />
              Campaign Details
            </CardTitle>
            <CardDescription className="text-slate-400">
              Fill in the details for your fundraising campaign
            </CardDescription>
          </CardHeader>

          <CardContent>
            <form onSubmit={handleSubmit} className="space-y-6">
              {/* Campaign ID */}
              <div className="space-y-2">
                <Label
                  htmlFor="campaignId"
                  className="text-slate-300 flex items-center gap-2"
                >
                  <Target className="w-4 h-4 text-[rgb(var(--solraiser-violet-400))]" />
                  Campaign ID
                </Label>
                <input
                  id="campaignId"
                  type="number"
                  value={campaignId}
                  onChange={(e) => setCampaignId(e.target.value)}
                  placeholder="Enter unique campaign ID (e.g., 1, 2, 3...)"
                  className="w-full px-4 py-3 bg-[rgb(var(--solraiser-slate-700))]/50 border border-white/10 rounded-xl text-white placeholder:text-slate-500 focus:outline-none focus:ring-2 focus:ring-[rgb(var(--solraiser-violet-500))]/50 focus:border-[rgb(var(--solraiser-violet-400))] transition-all"
                  required
                  min="0"
                  step="1"
                />
                <p className="text-xs text-slate-500">
                  A unique identifier for your campaign
                </p>
              </div>

              {/* Goal Amount */}
              <div className="space-y-2">
                <Label
                  htmlFor="goalAmount"
                  className="text-slate-300 flex items-center gap-2"
                >
                  <Target className="w-4 h-4 text-[rgb(var(--solraiser-fuchsia-400))]" />
                  Goal Amount (SOL)
                </Label>
                <input
                  id="goalAmount"
                  type="number"
                  value={goalAmount}
                  onChange={(e) => setGoalAmount(e.target.value)}
                  placeholder="Enter fundraising goal in SOL"
                  className="w-full px-4 py-3 bg-[rgb(var(--solraiser-slate-700))]/50 border border-white/10 rounded-xl text-white placeholder:text-slate-500 focus:outline-none focus:ring-2 focus:ring-[rgb(var(--solraiser-violet-500))]/50 focus:border-[rgb(var(--solraiser-violet-400))] transition-all"
                  required
                  min="0"
                  step="0.000000001"
                />
                <p className="text-xs text-slate-500">
                  The target amount you want to raise (in lamports)
                </p>
              </div>

              {/* Deadline */}
              <div className="space-y-2">
                <Label
                  htmlFor="deadline"
                  className="text-slate-300 flex items-center gap-2"
                >
                  <CalendarIcon className="w-4 h-4 text-[rgb(var(--solraiser-violet-400))]" />
                  Deadline
                </Label>
                <input
                  id="deadline"
                  type="datetime-local"
                  value={deadline}
                  onChange={(e) => setDeadline(e.target.value)}
                  className="w-full px-4 py-3 bg-[rgb(var(--solraiser-slate-700))]/50 border border-white/10 rounded-xl text-white placeholder:text-slate-500 focus:outline-none focus:ring-2 focus:ring-[rgb(var(--solraiser-violet-500))]/50 focus:border-[rgb(var(--solraiser-violet-400))] transition-all"
                  required
                />
                <p className="text-xs text-slate-500">
                  Campaign end date and time (must be in the future)
                </p>
              </div>

              {/* Metadata URL */}
              <div className="space-y-2">
                <Label
                  htmlFor="metadataUrl"
                  className="text-slate-300 flex items-center gap-2"
                >
                  <Link2Icon className="w-4 h-4 text-[rgb(var(--solraiser-fuchsia-400))]" />
                  Metadata URL
                </Label>
                <input
                  id="metadataUrl"
                  type="url"
                  value={metadataUrl}
                  onChange={(e) => setMetadataUrl(e.target.value)}
                  placeholder="https://example.com/metadata.json"
                  className="w-full px-4 py-3 bg-[rgb(var(--solraiser-slate-700))]/50 border border-white/10 rounded-xl text-white placeholder:text-slate-500 focus:outline-none focus:ring-2 focus:ring-[rgb(var(--solraiser-violet-500))]/50 focus:border-[rgb(var(--solraiser-violet-400))] transition-all"
                  required
                />
                <p className="text-xs text-slate-500">
                  URL to your campaign metadata (description, images, etc.)
                </p>
              </div>

              {/* Submit Button */}
              <div className="pt-4">
                <Button
                  type="submit"
                  disabled={isSubmitting}
                  className="w-full relative group bg-gradient-to-r from-[rgb(var(--solraiser-violet-600))] to-[rgb(var(--solraiser-fuchsia-600))] hover:from-[rgb(var(--solraiser-violet-500))] hover:to-[rgb(var(--solraiser-fuchsia-500))] text-white font-semibold py-6 rounded-xl shadow-lg hover:shadow-xl hover:shadow-[rgb(var(--solraiser-violet-500))]/50 transition-all duration-300"
                >
                  {/* Animated gradient border effect */}
                  <div className="absolute inset-0 rounded-xl bg-gradient-to-r from-[rgb(var(--solraiser-violet-400))] via-[rgb(var(--solraiser-fuchsia-400))] to-[rgb(var(--solraiser-violet-400))] opacity-0 group-hover:opacity-20 blur transition-opacity duration-300" />
                  
                  <span className="relative flex items-center justify-center gap-2">
                    <RocketIcon className="w-5 h-5" />
                    {isSubmitting ? "Creating Campaign..." : "Create Campaign"}
                  </span>
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>

        {/* Info Card */}
        <Card className="mt-6 bg-[rgb(var(--solraiser-violet-900))]/30 backdrop-blur-xl border-[rgb(var(--solraiser-violet-500))]/20">
          <CardContent className="pt-6">
            <div className="space-y-2">
              <h3 className="text-sm font-semibold text-[rgb(var(--solraiser-violet-300))]">
                Important Notes
              </h3>
              <ul className="text-xs text-slate-400 space-y-1 list-disc list-inside">
                <li>Campaign ID must be unique for your wallet</li>
                <li>Goal amount must be greater than 0</li>
                <li>Deadline must be set in the future</li>
                <li>Metadata URL should link to valid JSON containing campaign details</li>
                <li>Once created, campaign parameters cannot be changed</li>
              </ul>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}