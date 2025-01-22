use super::StakingInstance;
use super::User;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct EnterStaking<'info> {
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
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> EnterStaking<'info> {
    pub fn into_transfer_to_vault_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.user_lp_token_account.to_account_info(),
                to: self.gdtc_lp_in_account.to_account_info(),
                authority: self.authority.to_account_info(),
            },
        )
    }
}
