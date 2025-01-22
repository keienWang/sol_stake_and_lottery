use super::StakingInstance;
use super::User;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub authority: Signer<'info>, //签名用户
    #[account(mut)]
    pub staking_instance: Account<'info, StakingInstance>, //程序状态账户
    #[account(mut)]
    pub user_instance: Box<Account<'info, User>>, // 用户状态账户
    #[account(mut)]
    pub super_instance: Box<Account<'info, User>>, //上级状态账户
    #[account(mut)]
    pub user_super_gdtc_token_account: Box<Account<'info, TokenAccount>>, //上级的gdtc token账户
    #[account(mut)]
    pub user_gdtc_token_account: Account<'info, TokenAccount>, // 用户gdtc token账户
    #[account(mut)]
    pub gdtc_reward_out_account: Account<'info, TokenAccount>, //合约转出gdtc 的token账户
    
    /// CHECK: `pda_account` is a derived account from the program, and we ensure it's valid at runtime
    #[account(
        mut,
        seeds = [crate::LPTOKEN_SEED.as_ref()], 
        bump,
    )]
    pub pda_account: AccountInfo<'info>, //合约pda账户
    pub system_program: Program<'info, System>, //系统账户 programid
    pub token_program: Program<'info, Token>,   //token账户 可从sdk里导入
}
