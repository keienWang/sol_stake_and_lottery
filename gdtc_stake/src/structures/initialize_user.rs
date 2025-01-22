use super::{StakingInstance, User};
use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

#[derive(Accounts)]
pub struct InitializeUser<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub staking_instance: Account<'info, StakingInstance>,
    #[account(
        init,
        payer = authority,
        space = 8 + core::mem::size_of::<User>(),
        seeds = [
            crate::USER_SEED.as_ref(),
            staking_instance.key().as_ref(),
            authority.key().as_ref()
        ],
        bump,
    )]
    pub user_instance: Account<'info, User>,
    
    pub user_superior_token_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
}
