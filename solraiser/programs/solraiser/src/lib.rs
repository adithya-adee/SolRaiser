use anchor_lang::prelude::*;

declare_id!("62NbBCCxPfR83xtgw3AaxKGHyyDdxobrcCGzA7s7LFie");

#[program]
pub mod solraiser {
    use super::*;
    /// Creates a new fundraising campaign
    pub fn create_campaign(
        ctx: Context<CreateCampaign>,
        campaign_id: u64,
        goal_amount: u64,
        deadline: i64,
        metadata_url: String,
    ) -> Result<()> {

        require!(goal_amount > 0, ErrorCode::InvalidGoalAmount);
        require!(
            deadline > Clock::get()?.unix_timestamp,
            ErrorCode::InvalidDeadline
        );
        require!(
            metadata_url.len() <= Campaign::MAX_METADATA_URL_LEN,
            ErrorCode::MetadataUrlTooLong
        );

        let campaign = &mut ctx.accounts.campaign_account;
        campaign.creator_pubkey = ctx.accounts.creator.key();
        campaign.campaign_id = campaign_id;
        campaign.goal_amount = goal_amount;
        campaign.amount_raised = 0;
        campaign.deadline = deadline;
        campaign.metadata_url = metadata_url;
        campaign.is_withdrawn = false;
        campaign.withdrawn_amount = 0;

        msg!(
            "Campaign {} created successfully by {}",
            campaign_id,
            ctx.accounts.creator.key()
        );
        Ok(())
    }

    /// Donates funds to an active campaign
    /// Allows overfunding beyond goal (common crowdfunding behavior)
    pub fn donate(ctx: Context<Donate>, amount: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidAmount);

        let campaign = &ctx.accounts.campaign_account;

        require!(
            Clock::get()?.unix_timestamp < campaign.deadline,
            ErrorCode::CampaignExpired
        );

        let cpi_accounts = anchor_lang::system_program::Transfer {
            from: ctx.accounts.donor.to_account_info(),
            to: ctx.accounts.campaign_account.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.system_program.to_account_info(), cpi_accounts);
        anchor_lang::system_program::transfer(cpi_ctx, amount)?;

        let campaign = &mut ctx.accounts.campaign_account;
        campaign.amount_raised = campaign
            .amount_raised
            .checked_add(amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;

        msg!(
            "Donation of {} lamports received for campaign {} (total: {}/{})",
            amount,
            campaign.campaign_id,
            campaign.amount_raised,
            campaign.goal_amount
        );
        Ok(())
    }

    /// Withdraws funds from a successful campaign
    /// Withdraws ALL funds (including overfunding) to prevent locked lamports
    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign_account;

        require!(
            Clock::get()?.unix_timestamp > campaign.deadline,
            ErrorCode::CampaignStillActive
        );
        let campaign_lamports = campaign.to_account_info().lamports();
        let creator_lamports = ctx.accounts.creator.to_account_info().lamports();

        let rent = Rent::get()?;
        let min_rent = rent.minimum_balance(Campaign::LEN);
        let withdraw_amount = campaign_lamports
            .checked_sub(min_rent)
            .ok_or(ErrorCode::InsufficientFunds)?;

        require!(withdraw_amount > 0, ErrorCode::InsufficientFunds);

        **campaign.to_account_info().try_borrow_mut_lamports()? = min_rent;

        **ctx
            .accounts
            .creator
            .to_account_info()
            .try_borrow_mut_lamports()? = creator_lamports
            .checked_add(withdraw_amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;

        campaign.is_withdrawn = true;
        campaign.withdrawn_amount = withdraw_amount;

        msg!(
            "Withdrawal of {} lamports to creator (goal: {}, raised: {})",
            withdraw_amount,
            campaign.goal_amount,
            campaign.amount_raised
        );
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
        // Removed overfunding constraint - allows donations beyond goal (common crowdfunding behavior)
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
        bump,
        // Keep business logic constraints here, move time checks to require! for clarity
        constraint = campaign_account.amount_raised >= campaign_account.goal_amount @ ErrorCode::GoalNotReached,
        constraint = !campaign_account.is_withdrawn @ ErrorCode::AlreadyWithdrawn
    )]
    pub campaign_account: Account<'info, Campaign>,

    #[account(
        mut,
        constraint = creator.key() == campaign_account.creator_pubkey @ ErrorCode::UnauthorizedWithdraw
    )]
    pub creator: Signer<'info>,
}

#[account]
pub struct Campaign {
    pub creator_pubkey: Pubkey, // 32 bytes
    pub campaign_id: u64,       // 8 bytes
    pub goal_amount: u64,       // 8 bytes
    pub amount_raised: u64,     // 8 bytes (historical total - may exceed goal)
    pub deadline: i64,          // 8 bytes (i64 for timestamp)
    pub metadata_url: String,   // 4 + MAX_METADATA_URL_LEN bytes
    pub is_withdrawn: bool,     // 1 byte
    pub withdrawn_amount: u64,  // 8 bytes (actual amount withdrawn)
}

impl Campaign {
    pub const MAX_METADATA_URL_LEN: usize = 256;
    // Discriminator (8) + Pubkey (32) + u64*4 (32) + i64 (8) + String (4 + 256) + bool (1)
    pub const LEN: usize = 8 + 32 + 8 + 8 + 8 + 8 + 8 + 4 + Self::MAX_METADATA_URL_LEN + 1;
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
    #[msg("Unauthorized withdrawal - only campaign creator can withdraw")]
    UnauthorizedWithdraw,
    #[msg("Campaign has already reached its goal")]
    CampaignGoalReached,
    #[msg("Campaign deadline has passed")]
    CampaignExpired,
    #[msg("Campaign is still active, cannot withdraw yet")]
    CampaignStillActive,
    #[msg("Campaign goal has not been reached")]
    GoalNotReached,
    #[msg("Arithmetic overflow occurred")]
    ArithmeticOverflow,
    #[msg("Insufficient funds - withdrawal would violate rent exemption")]
    InsufficientFunds,
    #[msg("Campaign has already been withdrawn")]
    AlreadyWithdrawn,
}
