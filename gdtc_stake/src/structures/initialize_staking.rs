use super::{StakingInstance, StakingPool};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

#[derive(Accounts)]
pub struct InitializeStaking<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init, 
        seeds = [crate::STAKING_SEED.as_ref()],
        bump,
        space = 8 + core::mem::size_of::<StakingInstance>()+3 * core::mem::size_of::<StakingPool>(),
        payer = authority
    )]
    pub staking_instance: Account<'info, StakingInstance>,
    pub reward_token_mint: Account<'info, Mint>,
    pub staking_token_mint: Account<'info, Mint>,
    pub lp_token_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
}
