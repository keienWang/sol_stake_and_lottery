use super::{LotteryRound, LotteryState, UserLotteryState};
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount,transfer};
use anchor_lang::solana_program::program::invoke_signed;


#[derive(Accounts)]
#[instruction(round_number:u64)]
pub struct AdminClaimReward<'info> {

    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(mut,
        seeds = [crate::LOTTERY_SEED.as_ref()],
            bump)]
    pub lottery_state: Account<'info, LotteryState>, // 抽奖状态账户
    #[account(mut)]
    pub authority: Signer<'info>, //签名用户
    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(mut,
    seeds = [crate::LOTTERY_ROUND_SEED.as_ref(),&round_number.to_le_bytes()], // 动态轮次号,
        bump)]
    pub lottery_round: Account<'info, LotteryRound>,
    #[account(mut)]
    pub admin_gdtc_token_account: Account<'info, TokenAccount>, // 用户gdtc token账户
    #[account(mut)]
    pub gdtc_reward_out_account: Account<'info, TokenAccount>, //合约转出gdtc 的token账户
    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(
        mut,
        seeds = [crate::LPTOKEN_SEED.as_ref()], 
        bump,
    )]
    pub pda_account: AccountInfo<'info>, //合约pda账户
    pub system_program: Program<'info, System>, //系统账户 programid
    pub token_program: Program<'info, Token>,   //token账户 可从sdk里导入
}

impl<'info> AdminClaimReward<'info> {
    pub fn process(&mut self) -> Result<()> {
        
        
        Ok(())
    }

    
}
#[error_code]
pub enum ErrorCode {

    #[msg("The lottery round is  active.")]
    LotteryRoundActive, //本轮彩票还在进行中

    #[msg("Reward has already been claimed.")]
    RewardAlreadyClaimed,  // 奖励已经被领取

    #[msg("The user is not the winner of this lottery round.")]
    NotWinner, //非奖励赢家

    #[msg("User index is not match.")]
    UserIndexIsNotMatch,
}
