use super::LotteryState;
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

#[derive(Accounts)]
pub struct Initialize<'info> {
    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(
        init, 
        payer = authority, 
        space = 8+core::mem::size_of::<LotteryState>(),
        seeds = [crate::LOTTERY_SEED.as_ref()],
        bump
        )]
    pub lottery_state: Account<'info, LotteryState>, // 抽奖状态账户
    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(mut)]
    pub authority: Signer<'info>, // 合约管理员账户
    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(mut)]
    pub fund_foundation: AccountInfo<'info>, // 基金会账户
    pub reward_token_mint: Account<'info, Mint>, // 奖励代币 Mint 地址 (GDTC)
    pub staking_token_mint: Account<'info, Mint>, // 质押代币 Mint 地址 (LP)
    pub system_program: Program<'info, System>,  // 系统程序
}

impl<'info> Initialize<'info> {
    pub fn process(&mut self) -> Result<()> {
        let lottery_state = &mut self.lottery_state;
        // 设置抽奖合约的状态
        lottery_state.authority = self.authority.key();
        lottery_state.fund_foundation = self.fund_foundation.key();
        lottery_state.reward_token_mint = self.reward_token_mint.key();
        lottery_state.staking_token_mint = self.staking_token_mint.key();
        lottery_state.lottery_number = 0;
        Ok(())
    }
}
