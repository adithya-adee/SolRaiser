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
        campaign_account.amount_raised = 0;
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
        require!(amount > 0, ErrorCode::InvalidAmount);

        let cpi_accounts = anchor_lang::system_program::Transfer {
            from: ctx.accounts.donor.to_account_info(),
            to: ctx.accounts.campaign_account.to_account_info(),
        };

        let cpi_context =
            CpiContext::new(ctx.accounts.system_program.to_account_info(), cpi_accounts);

        anchor_lang::system_program::transfer(cpi_context, amount)?;

        let campaign_account = &mut ctx.accounts.campaign_account;
        campaign_account.amount_raised = campaign_account
            .amount_raised
            .checked_add(amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;

        msg!(
            "Donation of {} lamports received for campaign {}",
            amount,
            campaign_account.campaign_id
        );
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let campaign_account = &mut ctx.accounts.campaign_account;

        require!(
            campaign_account.amount_raised >= campaign_account.goal_amount,
            ErrorCode::GoalNotReached
        );

        require!(
            Clock::get()?.unix_timestamp > campaign_account.deadline,
            ErrorCode::CampaignStillActive
        );

        let withdraw_amount = campaign_account.goal_amount;

        // Get current lamports for safe transfer
        let campaign_lamports = campaign_account.to_account_info().lamports();
        let creator_lamports = ctx.accounts.creator.to_account_info().lamports();

        // Ensure campaign account maintains rent exemption after withdrawal
        let rent = Rent::get()?;
        let min_rent = rent.minimum_balance(Campaign::LEN);

        require!(
            campaign_lamports
                >= withdraw_amount
                    .checked_add(min_rent)
                    .ok_or(ErrorCode::ArithmeticOverflow)?,
            ErrorCode::InsufficientFunds
        );

        // Safe lamport transfer with explicit overflow/underflow checks
        **campaign_account
            .to_account_info()
            .try_borrow_mut_lamports()? = campaign_lamports
            .checked_sub(withdraw_amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;

        **ctx
            .accounts
            .creator
            .to_account_info()
            .try_borrow_mut_lamports()? = creator_lamports
            .checked_add(withdraw_amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;

        // Update campaign state
        campaign_account.amount_raised = campaign_account
            .amount_raised
            .checked_sub(withdraw_amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;

        msg!("Withdrawal of {} lamports to creator", withdraw_amount);
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
        constraint = campaign_account.amount_raised < campaign_account.goal_amount @ ErrorCode::CampaignGoalReached,
        constraint = Clock::get()?.unix_timestamp < campaign_account.deadline @ ErrorCode::CampaignExpired
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
}
