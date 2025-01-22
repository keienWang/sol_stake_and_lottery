use super::{LotteryRound, LotteryState, UserLotteryState};
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount,transfer};
use anchor_lang::solana_program::program::invoke_signed;


#[derive(Accounts)]
#[instruction(round_number:u64)]
pub struct ClaimReward<'info> {

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

        #[account(mut, seeds = [authority.key().as_ref()],bump)]
    pub user_lottery_state: Account<'info, UserLotteryState>, // 用户抽奖状态账户

    #[account(mut)]
    pub user_gdtc_token_account: Account<'info, TokenAccount>, // 用户gdtc token账户
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

impl<'info> ClaimReward<'info> {
    pub fn process(&mut self,bump_seed:u8 ,user_index :u64) -> Result<()> {
        
        let lottery_round = &mut self.lottery_round;
        let user_lottery_state = &mut self.user_lottery_state;

        let user_gdtc_token_account = &mut self.user_gdtc_token_account;
        let gdtc_reward_out_account = &mut self.gdtc_reward_out_account;

        if lottery_round.is_active {
            return Err(ErrorCode::LotteryRoundActive.into()); // 如果轮次未激活，则返回错误
        }
        if lottery_round.reward_claimed {
            return Err(ErrorCode::RewardAlreadyClaimed.into()); 
        }
        if lottery_round.winner != user_lottery_state.user_address {
            return Err(ErrorCode::NotWinner.into());  // 如果不是中奖者
        }

        if user_lottery_state.participated_rounds[user_index as usize].round_number != lottery_round.round_number {
            return Err(ErrorCode::UserIndexIsNotMatch.into()); // 如果已参与本轮，则返回错误
        }
         //  遍历 participated_rounds 找到对应轮次的记录
         let participated_round = user_lottery_state
         .participated_rounds
         .iter_mut()
         .find(|round| round.round_number == lottery_round.round_number);

        // 判断用户是否已经参与了本轮
        if let Some(round) = participated_round {
         if round.is_exist {
            lottery_round.reward_claimed = true;
         }
     }

     // 获取 PDA 签名者
     let signer_seeds: &[&[&[u8]]] = &[&[crate::LPTOKEN_SEED.as_ref(), &[bump_seed]]];

     // 生成从 GDTC 托管账户到用户 LP Token 账户的转账指令
     let transfer_instruction = spl_token::instruction::transfer(
         &self.token_program.key(),
         &self.gdtc_reward_out_account.key(),
         &self.user_gdtc_token_account.key(),
         &self.pda_account.key(),
         &[],
         10_000_000_000, //奖励10个gdtc
     )?;

     // 执行带签名的 CPI 调用
     invoke_signed(
         &transfer_instruction,
         &[
            self.token_program.to_account_info(),
            self.gdtc_reward_out_account.to_account_info(),
            self.user_gdtc_token_account.to_account_info(),
            self.pda_account.to_account_info(),
         ],
         signer_seeds,
     )?;

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
