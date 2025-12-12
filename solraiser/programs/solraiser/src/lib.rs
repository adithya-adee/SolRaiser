use anchor_lang::prelude::*;

declare_id!("62NbBCCxPfR83xtgw3AaxKGHyyDdxobrcCGzA7s7LFie");

#[program]
pub mod solraiser {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[account]
pub struct Campaign {
    pub creator_pubkey: Pubkey,
    pub goal_amount: u64,
    pub amount_raised: u64,
    pub deadline: u64,
    pub metadata_url: String,
}

impl Campaign {
    pub const MAX_METADATA_URL_LEN: usize = 256;
    pub const LEN: usize = 8 + 32 + 8 + 8 + 8 + 4 + Self::MAX_METADATA_URL_LEN;
}

#[account]
pub struct Donation {
    pub donor_pubkey: Pubkey,
    pub amount: u64,
    pub campaign_pubkey: Pubkey,
}

impl Donation {
    pub const LEN: usize = 8 + 32 + 8 + 32;
}
