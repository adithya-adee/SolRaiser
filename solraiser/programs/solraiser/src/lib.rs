use anchor_lang::prelude::*;

declare_id!("62NbBCCxPfR83xtgw3AaxKGHyyDdxobrcCGzA7s7LFie");

#[program]
pub mod solraiser {
    use super::*;
    pub fn create_campaign(
        ctx: Context<CreateCampaign>,
        campaign_id: u64,
        goal_amount: u64,
        deadline: i64,
        metadata_url: String,
    ) -> Result<()> {
        let campaign_account = &mut ctx.accounts.campaign_account;

        // Validations
        require!(goal_amount > 0, ErrorCode::InvalidGoalAmount);
        require!(
            deadline > Clock::get()?.unix_timestamp,
            ErrorCode::InvalidDeadline
        );
        require!(
            metadata_url.len() <= Campaign::MAX_METADATA_URL_LEN,
            ErrorCode::MetadataUrlTooLong
        );

        campaign_account.creator_pubkey = ctx.accounts.creator.key();
        campaign_account.campaign_id = campaign_id;
        campaign_account.goal_amount = goal_amount;
        campaign_account.amount_raised = 0; // Always start at 0
        campaign_account.deadline = deadline;
        campaign_account.metadata_url = metadata_url;

        msg!(
            "Campaign {} created successfully by {}",
            campaign_id,
            ctx.accounts.creator.key()
        );
        Ok(())
    }

    pub fn donate(ctx: Context<Donate>, amount: u64) -> Result<()> {
        let campaign_account = &mut ctx.accounts.campaign_account;

        require!(amount > 0, ErrorCode::InvalidAmount);

        campaign_account.amount_raised += amount;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let campaign_account = &mut ctx.accounts.campaign_account;

        require!(
            campaign_account.amount_raised >= campaign_account.goal_amount,
            ErrorCode::InvalidAmount
        );

        require!(
            campaign_account.deadline < Clock::get()?.unix_timestamp,
            ErrorCode::InvalidDeadline
        );

        campaign_account.amount_raised -= campaign_account.goal_amount;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(campaign_id: u64)]
pub struct CreateCampaign<'info> {
    #[account(
        init,
        payer = creator,
        space = Campaign::LEN,
        seeds = [b"campaign", creator.key().as_ref(), campaign_id.to_le_bytes().as_ref()],
        bump
    )]
    pub campaign_account: Account<'info, Campaign>,

    #[account(mut)]
    pub creator: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Donate<'info> {
    #[account(
        mut,
        seeds = [b"campaign", campaign_account.creator_pubkey.as_ref(), campaign_account.campaign_id.to_le_bytes().as_ref()],
        bump,
        constraint = campaign_account.amount_raised < campaign_account.goal_amount @ ErrorCode::UnauthorizedWithdraw
    )]
    pub campaign_account: Account<'info, Campaign>,

    #[account(mut)]
    pub donor: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [b"campaign", campaign_account.creator_pubkey.as_ref(), campaign_account.campaign_id.to_le_bytes().as_ref()],
        bump
    )]
    pub campaign_account: Account<'info, Campaign>,

    #[account(mut)]
    pub creator: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct Campaign {
    pub creator_pubkey: Pubkey, // 32 bytes
    pub campaign_id: u64,       // 8 bytes
    pub goal_amount: u64,       // 8 bytes
    pub amount_raised: u64,     // 8 bytes
    pub deadline: i64,          // 8 bytes (i64 for timestamp)
    pub metadata_url: String,   // 4 + MAX_METADATA_URL_LEN bytes
}

impl Campaign {
    pub const MAX_METADATA_URL_LEN: usize = 256;
    pub const LEN: usize = 8 + 32 + 8 + 8 + 8 + 8 + 4 + Self::MAX_METADATA_URL_LEN; // 8 (discriminator) + fields
}

// #[account]
// pub struct Donation {
//     pub donor_pubkey: Pubkey,
//     pub amount: u64,
//     pub campaign_pubkey: Pubkey,
// }

// impl Donation {
//     pub const LEN: usize = 8 + 32 + 8 + 32;
// }

#[error_code]
pub enum ErrorCode {
    #[msg("Goal amount must be greater than 0")]
    InvalidGoalAmount,
    #[msg("Deadline must be in the future")]
    InvalidDeadline,
    #[msg("Metadata URL exceeds maximum length")]
    MetadataUrlTooLong,
    #[msg("Amount must be greater than 0")]
    InvalidAmount,
    #[msg("Unauthorized withdrawal")]
    UnauthorizedWithdraw,
}
