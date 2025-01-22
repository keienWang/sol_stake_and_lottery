use super::{LotteryRound, LotteryState, UserLotteryState};
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount,transfer};
use anchor_lang::solana_program::program::invoke_signed;

use gdtc_stake::structures::{StakingInstance,User};
use gdtc_stake::program::GdtcStaking;
use gdtc_stake::cpi::accounts::CancelStaking;

#[derive(Accounts)]
#[instruction(round_number:u64)]
pub struct ClaimLP<'info> {

    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(mut,
        seeds = [crate::LOTTERY_SEED.as_ref()],
            bump)]
        pub lottery_state: Account<'info, LotteryState>, // 抽奖状态账户
        /// CHECK:` doc comment explaining why no checks through types are necessary.
        #[account(mut,
        seeds = [crate::LOTTERY_ROUND_SEED.as_ref(),&round_number.to_le_bytes()], // 动态轮次号,
        bump)]
        pub lottery_round: Account<'info, LotteryRound>,
        /// CHECK:` doc comment explaining why no checks through types are necessary.
        #[account(mut, seeds = [authority.key().as_ref()],bump)]
        pub user_lottery_state: Account<'info, UserLotteryState>, // 用户抽奖状态账户
        #[account(mut)]
        pub authority: Signer<'info>, //签名账户
        #[account(mut)]
        pub user_lp_token_account: Account<'info, TokenAccount>,
        #[account(mut)]
        pub gdtc_lp_in_account: Account<'info, TokenAccount>,
       
        pub system_program: Program<'info, System>, // 系统程序
        
        // 质押合约账户
        #[account(mut)]
        pub staking_instance: Account<'info, StakingInstance>, // 质押合约的状态账户
        #[account(mut)]
        pub user_instance: Account<'info, User>, //彩票合约指定期数的用户实例
        #[account(mut)] 
        pub gdtc_stake_lp_in_account: Account<'info, TokenAccount>,
        /// CHECK:` doc comment explaining why no checks through types are necessary.
        #[account(
            mut,
            seeds = [crate::LPTOKEN_SEED.as_ref(),&round_number.to_le_bytes()], 
            bump,
        )]
        pub pda_account: AccountInfo<'info>, //合约pda账户
        /// CHECK:` doc comment explaining why no checks through types are necessary.
        #[account(mut)]
        pub stake_pda_account:AccountInfo<'info>,
        pub staking_program: Program<'info, GdtcStaking>, // 质押合约
        pub token_program: Program<'info, Token>,
}

impl<'info> ClaimLP<'info> {
    pub fn process(&mut self,bump_seed:u8 ,user_index :u64,round_number:u64) -> Result<()> {
        
        let lottery_round = &mut self.lottery_round;
        let user_lottery_state = &mut self.user_lottery_state;

        let user_lp_token_account = &mut self.user_lp_token_account;

        let gdtc_lp_in_account = &mut self.gdtc_lp_in_account;

        if lottery_round.is_active {
            return Err(ErrorCode::LotteryRoundActive.into()); // 如果轮次正在进行
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

     //判断用户是否已经领取了lp
     if user_lottery_state.participated_rounds[user_index as usize].is_unstaked == true{
        return Err(ErrorCode::AlreadyUnstaked.into());
     }


     //修改彩票状态

     if lottery_round.is_unstake {

        let signer_seeds: &[&[&[u8]]] = &[&[crate::LPTOKEN_SEED.as_ref(),&round_number.to_le_bytes(), &[bump_seed]]];

        let cpi_program = self.staking_program.to_account_info();

        // Passing the necessary account(s) to the `BobAddOp` account struct in Bob program
        let cpi_account = CancelStaking {
            // bob_data_account: self.bob_data_account.to_account_info(),
            authority:self.pda_account.to_account_info(),
            staking_instance:self.staking_instance.to_account_info(),
            user_instance:self.user_instance.to_account_info(),
            user_lp_token_account:self.gdtc_lp_in_account.to_account_info(),
            gdtc_lp_in_account:self.gdtc_stake_lp_in_account.to_account_info(),
            pda_account:self.stake_pda_account.to_account_info(),
            system_program:self.system_program.to_account_info(),
            token_program:self.token_program.to_account_info(),
        };

        // Creates a `CpiContext` object using the new method
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_account,signer_seeds);

        let res = gdtc_stake::cpi::cancel_staking(cpi_ctx,0);

        // return an error if the CPI failed
        if !res.is_ok() {
            return Err(ErrorCode::CPIToStakeFailed.into())
        } 
        
    }

     //修改彩票轮状态
     lottery_round.total_lp = lottery_round
     .total_lp
     .checked_sub(1000_000_000)
     .ok_or(ErrorCode::Overflow)?;

     //修改用户状态
     user_lottery_state.participated_rounds[user_index as usize].is_unstaked =
            true;
       
     // 获取 PDA 签名者
     let signer_seeds: &[&[&[u8]]] = &[&[crate::LPTOKEN_SEED.as_ref(), &[bump_seed]]];

     // 生成从 GDTC 托管账户到用户 LP Token 账户的转账指令
     let transfer_instruction = spl_token::instruction::transfer(
         &self.token_program.key(),
         &self.gdtc_lp_in_account.key(),
         &self.user_lp_token_account.key(),
         &self.pda_account.key(),
         &[],
         1_000_000_000, //取消质押1lp
     )?;

     // 执行带签名的 CPI 调用
     invoke_signed(
         &transfer_instruction,
         &[
            self.token_program.to_account_info(),
            self.gdtc_lp_in_account.to_account_info(),
            self.user_lp_token_account.to_account_info(),
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

    #[msg("User has already unstaked.")]
    AlreadyUnstaked,

    #[msg("Arithmetic overflow occurred.")]
    Overflow,

    #[msg("User index is not match.")]
    UserIndexIsNotMatch,

    #[msg("CPI call to staking program failed.")]
    CPIToStakeFailed,
}
