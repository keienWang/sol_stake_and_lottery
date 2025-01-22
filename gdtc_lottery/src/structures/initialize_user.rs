use super::{ParticipatedRound, UserLotteryState};
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

#[derive(Accounts)]
pub struct InitializeUserLotteryState<'info> {
    #[account(
        init, 
        payer = authority, 
        space = 8 + core::mem::size_of::<UserLotteryState>() + 30 * core::mem::size_of::<ParticipatedRound>(),
        seeds = [authority.key().as_ref()],
        bump
    )]
    pub user_lottery_state: Account<'info, UserLotteryState>, // 用户抽奖状态账户
    #[account(mut)]
    pub authority: Signer<'info>, // 合约管理员账户 (即用户)
    pub system_program: Program<'info, System>, // 系统程序
}

impl<'info> InitializeUserLotteryState<'info> {
    pub fn process(&mut self) -> Result<()> {
        let user_lottery_state = &mut self.user_lottery_state;

        // 设置 UserLotteryState 账户的信息
        user_lottery_state.user_address = self.authority.key(); // 用户地址
                                                                // 初始化 participated_rounds 数组中的每个轮次
        for participated_round in user_lottery_state.participated_rounds.iter_mut() {
            participated_round.round_number = 0; // 初始轮次号为 0
            participated_round.is_unstaked = false; // 初始时未解除质押
            participated_round.is_exist = false;
        }

        Ok(())
    }
}
