use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
    token::{Mint, Token, TokenAccount, Transfer, transfer},
    associated_token::AssociatedToken,
};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod cfish_contract {
    use super::*;

    // NFT Core Program: mint_nft instruction (simplified version)
    pub fn mint_nft(
        ctx: Context<MintNft>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        // Simplified NFT minting without Metaplex dependency
        // In production, this would integrate with Metaplex Token Metadata
        
        // Store NFT metadata in our custom account
        ctx.accounts.nft_metadata.name = name;
        ctx.accounts.nft_metadata.symbol = symbol;
        ctx.accounts.nft_metadata.uri = uri;
        ctx.accounts.nft_metadata.mint = ctx.accounts.mint.key();
        ctx.accounts.nft_metadata.creator = ctx.accounts.mint_authority.key();
        
        msg!("NFT minted successfully: {}", ctx.accounts.nft_metadata.key());
        Ok(())
    }

    // Marketplace Program: list_nft instruction
    pub fn list_nft(
        ctx: Context<ListNft>,
        price: u64,
    ) -> Result<()> {
        // Transfer NFT from seller to escrow account
        let cpi_accounts = Transfer {
            from: ctx.accounts.seller_nft_token_account.to_account_info(),
            to: ctx.accounts.escrow_nft_token_account.to_account_info(),
            authority: ctx.accounts.seller.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, 1)?; // Transfer 1 NFT

        // Set the listing details
        ctx.accounts.listing.seller = ctx.accounts.seller.key();
        ctx.accounts.listing.nft_mint = ctx.accounts.nft_mint.key();
        ctx.accounts.listing.price = price;
        ctx.accounts.listing.escrow_nft_token_account = ctx.accounts.escrow_nft_token_account.key();
        ctx.accounts.listing.escrow_authority = ctx.accounts.escrow_authority.key();
        ctx.accounts.listing.is_sold = false;

        msg!("NFT listed for sale at price: {}", price);
        Ok(())
    }

    // Marketplace Program: buy_nft instruction
    pub fn buy_nft(
        ctx: Context<BuyNft>,
    ) -> Result<()> {
        let listing = &ctx.accounts.listing;
        
        // Transfer SOL from buyer to seller
        let cpi_accounts = system_program::Transfer {
            from: ctx.accounts.buyer.to_account_info(),
            to: ctx.accounts.seller.to_account_info(),
        };
        let cpi_program = ctx.accounts.system_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        system_program::transfer(cpi_ctx, listing.price)?;

        // Transfer NFT from escrow to buyer
        let seeds = &[
            b"escrow_authority",
            ctx.accounts.listing.nft_mint.as_ref(),
            &[ctx.accounts.escrow_authority.bump]
        ];
        let signer = &[&seeds[..]];
        
        let cpi_accounts = Transfer {
            from: ctx.accounts.escrow_nft_token_account.to_account_info(),
            to: ctx.accounts.buyer_nft_token_account.to_account_info(),
            authority: ctx.accounts.escrow_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        transfer(cpi_ctx, 1)?; // Transfer 1 NFT

        // Mark listing as sold
        ctx.accounts.listing.is_sold = true;

        msg!("NFT purchased successfully");
        Ok(())
    }

    // Staking & Governance Program: stake instruction
    pub fn stake(
        ctx: Context<Stake>,
        amount: u64,
        duration_days: u64,
    ) -> Result<()> {
        // Transfer CFISH tokens from staker to stake account
        let cpi_accounts = Transfer {
            from: ctx.accounts.staker_token_account.to_account_info(),
            to: ctx.accounts.stake_account.to_account_info(),
            authority: ctx.accounts.staker.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, amount)?;

        // Update stake entry
        ctx.accounts.stake_entry.staker = ctx.accounts.staker.key();
        ctx.accounts.stake_entry.amount = ctx.accounts.stake_entry.amount.checked_add(amount).unwrap();
        ctx.accounts.stake_entry.stake_start_time = Clock::get()?.unix_timestamp;
        ctx.accounts.stake_entry.duration_days = duration_days;
        ctx.accounts.stake_entry.claimed_rewards = 0;

        msg!("Staked {} CFISH tokens for {} days", amount, duration_days);
        Ok(())
    }

    // Staking & Governance Program: unstake instruction
    pub fn unstake(
        ctx: Context<Unstake>,
    ) -> Result<()> {
        // Calculate rewards (simplified calculation)
        let current_time = Clock::get()?.unix_timestamp;
        let elapsed_days = (current_time - ctx.accounts.stake_entry.stake_start_time) / (24 * 60 * 60);
        let rewards = (ctx.accounts.stake_entry.amount as f64 * 0.01 * elapsed_days as f64) as u64; // 1% daily reward

        // Transfer CFISH tokens + rewards from stake account to staker
        let staker_key = ctx.accounts.staker.key();
        let seeds = &[
            b"stake_authority",
            staker_key.as_ref(),
            &[ctx.accounts.stake_authority.bump]
        ];
        let signer = &[&seeds[..]];
        
        let total_amount = ctx.accounts.stake_entry.amount.checked_add(rewards).unwrap();
        let cpi_accounts = Transfer {
            from: ctx.accounts.stake_account.to_account_info(),
            to: ctx.accounts.staker_token_account.to_account_info(),
            authority: ctx.accounts.stake_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        transfer(cpi_ctx, total_amount)?;

        // Reset stake entry
        ctx.accounts.stake_entry.amount = 0;
        ctx.accounts.stake_entry.claimed_rewards = ctx.accounts.stake_entry.claimed_rewards.checked_add(rewards).unwrap();

        msg!("Unstaked {} CFISH tokens with {} rewards", ctx.accounts.stake_entry.amount, rewards);
        Ok(())
    }

    // Trading Mining: distribute_reward instruction
    pub fn distribute_reward(
        ctx: Context<DistributeReward>,
        reward_amount: u64,
    ) -> Result<()> {
        // Check daily limit (simplified)
        let current_day = Clock::get()?.unix_timestamp / (24 * 60 * 60);
        if ctx.accounts.reward_tracker.last_reward_day == current_day {
            ctx.accounts.reward_tracker.daily_count += 1;
            require!(ctx.accounts.reward_tracker.daily_count <= 5, CustomError::DailyLimitExceeded);
        } else {
            ctx.accounts.reward_tracker.last_reward_day = current_day;
            ctx.accounts.reward_tracker.daily_count = 1;
        }

        // Create vesting entry for reward
        ctx.accounts.vesting_entry.beneficiary = ctx.accounts.user.key();
        ctx.accounts.vesting_entry.total_amount = reward_amount;
        ctx.accounts.vesting_entry.released_amount = 0;
        ctx.accounts.vesting_entry.start_time = Clock::get()?.unix_timestamp + (24 * 60 * 60); // Start tomorrow
        ctx.accounts.vesting_entry.duration = 180 * 24 * 60 * 60; // 180 days

        msg!("Reward of {} CFISH distributed to user", reward_amount);
        Ok(())
    }

    // Governance: create_proposal instruction
    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        title: String,
        description: String,
        voting_period: i64,
    ) -> Result<()> {
        ctx.accounts.proposal.proposer = ctx.accounts.proposer.key();
        ctx.accounts.proposal.title = title;
        ctx.accounts.proposal.description = description;
        ctx.accounts.proposal.start_time = Clock::get()?.unix_timestamp;
        ctx.accounts.proposal.end_time = Clock::get()?.unix_timestamp + voting_period;
        ctx.accounts.proposal.yes_votes = 0;
        ctx.accounts.proposal.no_votes = 0;
        ctx.accounts.proposal.executed = false;
        ctx.accounts.proposal.bump = ctx.bumps.proposal;
        msg!("Proposal created: {}", ctx.accounts.proposal.title);
        Ok(())
    }

    // Governance: vote instruction
    pub fn vote(
        ctx: Context<Vote>,
        vote_yes: bool,
    ) -> Result<()> {
        let voting_power = ctx.accounts.stake_entry.amount; // Voting power based on staked amount
        
        if vote_yes {
            ctx.accounts.proposal.yes_votes = ctx.accounts.proposal.yes_votes.checked_add(voting_power).unwrap();
        } else {
            ctx.accounts.proposal.no_votes = ctx.accounts.proposal.no_votes.checked_add(voting_power).unwrap();
        }

        ctx.accounts.vote_record.voter = ctx.accounts.voter.key();
        ctx.accounts.vote_record.proposal = ctx.accounts.proposal.key();
        ctx.accounts.vote_record.vote_yes = vote_yes;
        ctx.accounts.vote_record.voting_power = voting_power;

        msg!("Vote cast: {} with power {}", if vote_yes { "YES" } else { "NO" }, voting_power);
        Ok(())
    }

    // Tokenomics: release_vested_reward instruction
    pub fn release_vested_reward(
        ctx: Context<ReleaseVestedReward>,
    ) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let vesting_entry = &mut ctx.accounts.vesting_entry;

        require!(current_time >= vesting_entry.start_time, CustomError::VestingNotStarted);

        let total_duration = vesting_entry.duration;
        let elapsed_duration = current_time - vesting_entry.start_time;

        let vested_amount = (vesting_entry.total_amount as f64 * (elapsed_duration as f64 / total_duration as f64)) as u64;
        let amount_to_release = vested_amount.checked_sub(vesting_entry.released_amount).unwrap();

        require!(amount_to_release > 0, CustomError::NoRewardsToRelease);

        // Transfer vested tokens from authority to beneficiary
        let authority_key = ctx.accounts.authority.key();
        let seeds = &[
            b"authority",
            authority_key.as_ref(),
            &[ctx.accounts.authority.bump]
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.authority_token_account.to_account_info(),
            to: ctx.accounts.beneficiary_token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        transfer(cpi_ctx, amount_to_release)?;

        vesting_entry.released_amount = vesting_entry.released_amount.checked_add(amount_to_release).unwrap();

        msg!("Released {} vested CFISH tokens", amount_to_release);
        Ok(())
    }
}

// Account structures for mint_nft instruction
#[derive(Accounts)]
pub struct MintNft<
    'info
> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub mint_authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 32 + 4 + 100 + 4 + 10 + 4 + 200, // Discriminator + mint + creator + name + symbol + uri
        seeds = [b"nft_metadata", mint.key().as_ref()],
        bump
    )]
    pub nft_metadata: Account<'info, NftMetadata>,
    pub system_program: Program<'info, System>,
}

// Account structures for list_nft instruction
#[derive(Accounts)]
#[instruction(price: u64)]
pub struct ListNft<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(mut)]
    pub nft_mint: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = seller
    )]
    pub seller_nft_token_account: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = seller,
        token::mint = nft_mint,
        token::authority = escrow_authority,
        seeds = [b"escrow", nft_mint.key().as_ref()],
        bump
    )]
    pub escrow_nft_token_account: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = seller,
        space = 8 + 32 + 32 + 8 + 32 + 32 + 1, // Discriminator + seller + nft_mint + price + escrow_nft_token_account + escrow_authority + is_sold
        seeds = [b"listing", nft_mint.key().as_ref()],
        bump
    )]
    pub listing: Account<'info, Listing>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: This is the PDA authority for the escrow account
    #[account(
        init,
        payer = seller,
        seeds = [b"escrow_authority", nft_mint.key().as_ref()],
        bump,
        space = 8 + 1 // Discriminator + bump
    )]
    pub escrow_authority: Account<'info, EscrowAuthority>,
}

// Account structures for buy_nft instruction
#[derive(Accounts)]
pub struct BuyNft<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    /// CHECK: The seller's account is validated by the listing account
    pub seller: AccountInfo<'info>,
    #[account(mut,
        constraint = listing.nft_mint == nft_mint.key(),
        constraint = listing.is_sold == false,
        has_one = nft_mint,
        has_one = escrow_nft_token_account,
        has_one = escrow_authority
    )]
    pub listing: Account<'info, Listing>,
    #[account(mut)]
    pub nft_mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = nft_mint,
        associated_token::authority = buyer
    )]
    pub buyer_nft_token_account: Account<'info, TokenAccount>,
    #[account(mut,
        token::mint = nft_mint,
        token::authority = escrow_authority,
        seeds = [b"escrow", nft_mint.key().as_ref()],
        bump
    )]
    pub escrow_nft_token_account: Account<'info, TokenAccount>,
    #[account(
        seeds = [b"escrow_authority", nft_mint.key().as_ref()],
        bump
    )]
    pub escrow_authority: Account<'info, EscrowAuthority>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// Account structures for stake instruction
#[derive(Accounts)]
#[instruction(amount: u64, duration_days: u64)]
pub struct Stake<'info> {
    #[account(mut)]
    pub staker: Signer<'info>,
    #[account(mut)]
    pub cfish_mint: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = cfish_mint,
        associated_token::authority = staker
    )]
    pub staker_token_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = staker,
        token::mint = cfish_mint,
        token::authority = stake_authority,
        seeds = [b"stake_account", staker.key().as_ref()],
        bump
    )]
    pub stake_account: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = staker,
        seeds = [b"stake_authority", staker.key().as_ref()],
        bump,
        space = 8 + 1 // Discriminator + bump
    )]
    pub stake_authority: Account<'info, StakeAuthority>,
    #[account(
        init_if_needed,
        payer = staker,
        space = 8 + 32 + 8 + 8 + 8 + 8, // Discriminator + staker + amount + stake_start_time + duration_days + claimed_rewards
        seeds = [b"stake_entry", staker.key().as_ref()],
        bump
    )]
    pub stake_entry: Account<'info, StakeEntry>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// Account structures for unstake instruction
#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub staker: Signer<'info>,
    #[account(mut)]
    pub cfish_mint: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = cfish_mint,
        associated_token::authority = staker
    )]
    pub staker_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = cfish_mint,
        token::authority = stake_authority,
        seeds = [b"stake_account", staker.key().as_ref()],
        bump
    )]
    pub stake_account: Account<'info, TokenAccount>,
    #[account(
        seeds = [b"stake_authority", staker.key().as_ref()],
        bump
    )]
    pub stake_authority: Account<'info, StakeAuthority>,
    #[account(
        mut,
        constraint = stake_entry.staker == staker.key(),
        seeds = [b"stake_entry", staker.key().as_ref()],
        bump
    )]
    pub stake_entry: Account<'info, StakeEntry>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// Account structures for distribute_reward instruction
#[derive(Accounts)]
pub struct DistributeReward<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub cfish_mint: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = cfish_mint,
        associated_token::authority = user
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 8 + 8, // Discriminator + last_reward_day + daily_count
        seeds = [b"reward_tracker", user.key().as_ref()],
        bump
    )]
    pub reward_tracker: Account<'info, RewardTracker>,
    #[account(
        init,
        payer = user,
        space = 8 + 32 + 8 + 8 + 8 + 8, // Discriminator + beneficiary + total_amount + released_amount + start_time + duration
        seeds = [b"vesting_entry", user.key().as_ref(), cfish_mint.key().as_ref()],
        bump
    )]
    pub vesting_entry: Account<'info, VestingEntry>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// Account structures for create_proposal instruction
#[derive(Accounts)]
#[instruction(title: String, description: String, voting_period: i64)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    #[account(
        init,
        payer = proposer,
        space = 8 + 32 + 256 + 1024 + 8 + 8 + 8 + 8 + 1, // Discriminator + proposer + title + description + start_time + end_time + yes_votes + no_votes + executed
        seeds = [b"proposal", proposer.key().as_ref(), title.as_bytes()],
        bump
    )]
    pub proposal: Account<'info, Proposal>,
    pub system_program: Program<'info, System>,
}

// Account structures for vote instruction
#[derive(Accounts)]
pub struct Vote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,
    #[account(mut,
        constraint = proposal.end_time > Clock::get()?.unix_timestamp,
        constraint = proposal.proposer == proposal.proposer,
    )]
    pub proposal: Account<'info, Proposal>,
    #[account(mut,
        constraint = stake_entry.staker == voter.key(),
        seeds = [b"stake_entry", voter.key().as_ref()],
        bump
    )]
    pub stake_entry: Account<'info, StakeEntry>,
    #[account(
        init,
        payer = voter,
        space = 8 + 32 + 32 + 1 + 8, // Discriminator + voter + proposal + vote_yes + voting_power
        seeds = [b"vote_record", proposal.key().as_ref(), voter.key().as_ref()],
        bump
    )]
    pub vote_record: Account<'info, VoteRecord>,
    pub system_program: Program<'info, System>,
}

// Account structures for release_vested_reward instruction
#[derive(Accounts)]
pub struct ReleaseVestedReward<'info> {
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    #[account(mut)]
    pub cfish_mint: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = cfish_mint,
        associated_token::authority = beneficiary
    )]
    pub beneficiary_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"vesting_entry", beneficiary.key().as_ref(), cfish_mint.key().as_ref()],
        bump,
        has_one = beneficiary
    )]
    pub vesting_entry: Account<'info, VestingEntry>,
    #[account(
        seeds = [b"authority"],
        bump
    )]
    pub authority: Account<'info, Authority>,
    #[account(
        mut,
        token::mint = cfish_mint,
        token::authority = authority,
        seeds = [b"authority_token_account", cfish_mint.key().as_ref()],
        bump
    )]
    pub authority_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

// Data structures

#[account]
pub struct NftMetadata {
    pub mint: Pubkey,
    pub creator: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

#[account]
pub struct Listing {
    pub seller: Pubkey,
    pub nft_mint: Pubkey,
    pub price: u64,
    pub escrow_nft_token_account: Pubkey,
    pub escrow_authority: Pubkey,
    pub is_sold: bool,
}

#[account]
pub struct EscrowAuthority {
    pub bump: u8,
}

#[account]
pub struct StakeEntry {
    pub staker: Pubkey,
    pub amount: u64,
    pub stake_start_time: i64,
    pub duration_days: u64,
    pub claimed_rewards: u64,
}

#[account]
pub struct StakeAuthority {
    pub bump: u8,
}

#[account]
pub struct RewardTracker {
    pub last_reward_day: i64,
    pub daily_count: u64,
}

#[account]
pub struct VestingEntry {
    pub beneficiary: Pubkey,
    pub total_amount: u64,
    pub released_amount: u64,
    pub start_time: i64,
    pub duration: i64,
}

#[account]
pub struct Proposal {
    pub proposer: Pubkey,
    pub title: String,
    pub description: String,
    pub start_time: i64,
    pub end_time: i64,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub executed: bool,
    pub bump: u8,
}

#[account]
pub struct VoteRecord {
    pub voter: Pubkey,
    pub proposal: Pubkey,
    pub vote_yes: bool,
    pub voting_power: u64,
}

#[account]
pub struct Authority {
    pub bump: u8,
}

#[error_code]
pub enum CustomError {
    #[msg("Daily reward limit exceeded")]
    DailyLimitExceeded,
    #[msg("Vesting has not started yet")]
    VestingNotStarted,
    #[msg("No rewards to release")]
    NoRewardsToRelease,
}


