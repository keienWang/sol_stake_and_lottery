use super::StakingInstance;
use super::User;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

#[derive(Accounts)]
pub struct CancelStaking<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub staking_instance: Account<'info, StakingInstance>,
    #[account(mut)]
    pub user_instance: Account<'info, User>,
    #[account(mut)]
    pub user_lp_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub gdtc_lp_in_account: Account<'info, TokenAccount>,

    /// CHECK: `pda_account` is a derived account from the program, and we ensure it's valid at runtime
    #[account(
        mut,
        seeds = [crate::LPTOKEN_SEED.as_ref()], 
        bump,
    )]
    pub pda_account: AccountInfo<'info>, // PDA 账户
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}
