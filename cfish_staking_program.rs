use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, Transfer};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"); // Placeholder Program ID

#[program]
pub mod cfish_contract {
    use super::*;

    // ... (previous instructions like mint_nft, list_nft, buy_nft will be here)

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
        anchor_spl::token::transfer(cpi_ctx, amount)?;

        // Update stake entry
        ctx.accounts.stake_entry.staker = ctx.accounts.staker.key();
        ctx.accounts.stake_entry.amount = ctx.accounts.stake_entry.amount.checked_add(amount).unwrap();
        ctx.accounts.stake_entry.stake_start_time = Clock::get()?.unix_timestamp;
        ctx.accounts.stake_entry.duration_days = duration_days;
        ctx.accounts.stake_entry.claimed_rewards = 0;

        Ok(())
    }

    pub fn unstake(
        ctx: Context<Unstake>,
    ) -> Result<()> {
        // Calculate rewards (simplified for now)
        let current_time = Clock::get()?.unix_timestamp;
        let elapsed_days = (current_time - ctx.accounts.stake_entry.stake_start_time) / (24 * 60 * 60);
        let rewards = (ctx.accounts.stake_entry.amount as f64 * 0.01 * elapsed_days as f64) as u64; // Example: 1% daily reward

        // Transfer CFISH tokens from stake account to staker
        let cpi_accounts = Transfer {
            from: ctx.accounts.stake_account.to_account_info(),
            to: ctx.accounts.staker_token_account.to_account_info(),
            authority: ctx.accounts.stake_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let seeds = &[b"stake_authority", ctx.accounts.staker.key().as_ref(), &[ctx.accounts.stake_authority.bump]];
        let signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        anchor_spl::token::transfer(cpi_ctx, ctx.accounts.stake_entry.amount.checked_add(rewards).unwrap())?;

        // Close stake account and stake entry
        // anchor_lang::system_program::close_account(CpiContext::new(ctx.accounts.system_program.to_account_info(), CloseAccount { account: ctx.accounts.stake_account.to_account_info(), destination: ctx.accounts.staker.to_account_info(), authority: ctx.accounts.stake_authority.to_account_info() }))?;
        // ctx.accounts.stake_entry.close(ctx.accounts.staker.to_account_info())?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(amount: u64, duration_days: u64)]
pub struct Stake<\'info> {
    #[account(mut)]
    pub staker: Signer<\'info>,
    #[account(mut)]
    pub cfish_mint: Account<\'info, Mint>,
    #[account(mut,
        associated_token::mint = cfish_mint,
        associated_token::authority = staker
    )]
    pub staker_token_account: Account<\'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = staker,
        token::mint = cfish_mint,
        token::authority = stake_authority,
        seeds = [b"stake_account", staker.key().as_ref()],
        bump
    )]
    pub stake_account: Account<\'info, TokenAccount>,
    /// CHECK: This is the PDA authority for the stake account
    #[account(
        seeds = [b"stake_authority", staker.key().as_ref()],
        bump
    )]
    pub stake_authority: AccountInfo<\'info>,
    #[account(
        init_if_needed,
        payer = staker,
        space = 8 + 32 + 8 + 8 + 8 + 8, // Discriminator + staker + amount + stake_start_time + duration_days + claimed_rewards
        seeds = [b"stake_entry", staker.key().as_ref()],
        bump
    )]
    pub stake_entry: Account<\'info, StakeEntry>,
    pub token_program: Program<\'info, Token>,
    pub system_program: Program<\'info, System>,
    pub rent: Sysvar<\'info, Rent>,
}

#[derive(Accounts)]
pub struct Unstake<\'info> {
    #[account(mut)]
    pub staker: Signer<\'info>,
    #[account(mut)]
    pub cfish_mint: Account<\'info, Mint>,
    #[account(mut,
        associated_token::mint = cfish_mint,
        associated_token::authority = staker
    )]
    pub staker_token_account: Account<\'info, TokenAccount>,
    #[account(mut,
        token::mint = cfish_mint,
        token::authority = stake_authority,
        seeds = [b"stake_account", staker.key().as_ref()],
        bump
    )]
    pub stake_account: Account<\'info, TokenAccount>,
    /// CHECK: This is the PDA authority for the stake account
    #[account(
        seeds = [b"stake_authority", staker.key().as_ref()],
        bump
    )]
    pub stake_authority: AccountInfo<\'info>,
    #[account(mut,
        has_one = staker,
        seeds = [b"stake_entry", staker.key().as_ref()],
        bump
    )]
    pub stake_entry: Account<\'info, StakeEntry>,
    pub token_program: Program<\'info, Token>,
    pub system_program: Program<\'info, System>,
}

#[account]
pub struct StakeEntry {
    pub staker: Pubkey,
    pub amount: u64,
    pub stake_start_time: i64,
    pub duration_days: u64,
    pub claimed_rewards: u64,
}


