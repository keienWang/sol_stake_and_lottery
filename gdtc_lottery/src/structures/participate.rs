use super::{LotteryRound, LotteryState, UserLotteryState};
use anchor_lang::prelude::*;
use solana_program::hash::{hash, Hash};
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer,transfer};
use gdtc_stake::structures::{StakingInstance,User};
use gdtc_stake::program::GdtcStaking;
use gdtc_stake::cpi::accounts::EnterStaking;
use anchor_lang::solana_program::program::invoke_signed;


#[derive(Accounts)]
#[instruction(round_number:u64)]
pub struct Participate<'info> {
    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(mut,
    seeds = [crate::LOTTERY_SEED.as_ref()],
        bump)]
    pub lottery_state: Account<'info, LotteryState>, // 抽奖状态账户
    #[account(mut,
    seeds = [crate::LOTTERY_ROUND_SEED.as_ref(),&round_number.to_le_bytes()], // 动态轮次号,
    bump)]
    pub lottery_round: Account<'info, LotteryRound>,
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
    pub staking_program: Program<'info, GdtcStaking>, // 质押合约
    pub token_program: Program<'info, Token>,
}

impl<'info> Participate<'info> {
    pub fn process(&mut self,bump_seed:u8 , user_index: u64,round_number :u64) -> Result<()> {
        let lottery_state = &mut self.lottery_state;
        let lottery_round = &mut self.lottery_round;
        let user_lottery_state = &mut self.user_lottery_state;
        let user_lp_token_account = &mut self.user_lp_token_account;
        let gdtc_lp_in_account = &mut self.gdtc_lp_in_account;
        if lottery_state.lottery_number != lottery_round.round_number {
            return Err(ErrorCode::LotteryRoundNumberMismatch.into());
        };
        //  遍历 participated_rounds 找到对应轮次的记录
        let participated_round = user_lottery_state
            .participated_rounds
            .iter_mut()
            .find(|round| round.round_number == lottery_round.round_number);

        // 判断用户是否已经参与了本轮
        if let Some(round) = participated_round {
            if round.is_exist {
                return Err(ErrorCode::AlreadyParticipated.into()); // 如果已参与本轮，则返回错误
            }
        }
        if user_lottery_state.participated_rounds[user_index as usize].is_exist {
            return Err(ErrorCode::AlreadyParticipated.into()); // 如果已参与本轮，则返回错误
        }

        if !lottery_round.is_active {
            return Err(ErrorCode::LotteryRoundNotActive.into()); // 如果轮次未激活，则返回错误
        }

        //区块hash无法拿到
        let clock = Clock::get()?;

        // 拼接用户地址和区块 slot（注意：你也可以加入其他字段）
        let user_address_bytes = &self.authority.key().to_bytes();
        let slot_bytes = clock.slot.to_le_bytes();

        // 合并用户地址和 slot 信息
        let mut seed = Vec::new();
        seed.extend_from_slice(&user_address_bytes[..]);
        seed.extend_from_slice(&slot_bytes);

        let block_hash = hash(&seed); // 使用区块高度（slot）来生成伪随机数
        let random_number = u64::from_le_bytes(block_hash.to_bytes()[0..8].try_into().unwrap());

        msg!(
            "User Address: {:?} Generated Hash: {:?}",
            &self.authority.key(),
            random_number
        );
        let lp_number = 1000_000_000;
        // 检查用户 LP Token 账户余额是否足够
        if user_lp_token_account.amount < lp_number {
            return Err(ErrorCode::TokenAccountBalanceInsufficient.into());
        }

        if random_number as u128 > lottery_round.max_hash_values {
            lottery_round.max_hash_values = random_number as u128;
            lottery_round.winner = self.authority.key();
        }
        //修改彩票轮状态
        lottery_round.total_lp = lottery_round
            .total_lp
            .checked_add(1000_000_000)
            .ok_or(ErrorCode::Overflow)?;

        //修改用户状态
        user_lottery_state.participated_rounds[user_index as usize].round_number =
            lottery_round.round_number;
        user_lottery_state.participated_rounds[user_index as usize].is_exist = true;

        

        let mut is_stake = false;
        if lottery_round.total_lp == 50 {
            lottery_round.round_end_time = Some(Clock::get()?.unix_timestamp);
            lottery_round.is_active = false;
            lottery_state.lottery_number = lottery_state
                .lottery_number
                .checked_add(1)
                .ok_or(ErrorCode::Overflow)?;
            lottery_round.is_unstake = false;
            is_stake = true;
        }



        //转入lp
        transfer(self.into_transfer_to_vault_context(), lp_number)?;
        

        
        // 获取 PDA 签名者
       
        let signer_seeds: &[&[&[u8]]] = &[&[crate::LPTOKEN_SEED.as_ref(),&round_number.to_le_bytes(), &[bump_seed]]];

        let cpi_program = self.staking_program.to_account_info();

        // Passing the necessary account(s) to the `BobAddOp` account struct in Bob program
        let cpi_account = EnterStaking {
            // bob_data_account: self.bob_data_account.to_account_info(),
            authority:self.pda_account.to_account_info(),
            staking_instance:self.staking_instance.to_account_info(),
            user_instance:self.user_instance.to_account_info(),
            user_lp_token_account:self.gdtc_lp_in_account.to_account_info(),
            gdtc_lp_in_account:self.gdtc_stake_lp_in_account.to_account_info(),
            system_program:self.system_program.to_account_info(),
            token_program:self.token_program.to_account_info(),
        };

        // Creates a `CpiContext` object using the new method
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_account,signer_seeds);

        let res = gdtc_stake::cpi::enter_staking(cpi_ctx, 5000_000_000,0,0);

        // return an error if the CPI failed
        if res.is_ok() {
            return Ok(());
        } else {
            return Err(ErrorCode::CPIToStakeFailed.into())
        }
        
    }

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

#[error_code]
pub enum ErrorCode {
    #[msg("The lottery round number does not match the lottery state number.")]
    LotteryRoundNumberMismatch,

    #[msg("User has already participated in this lottery round.")]
    AlreadyParticipated,

    #[msg("Arithmetic overflow occurred.")]
    Overflow,

    #[msg("The lottery round is not active.")]
    LotteryRoundNotActive,

    #[msg("Insufficient token account balance.")]
    TokenAccountBalanceInsufficient,

    #[msg("CPI call to staking program failed.")]
    CPIToStakeFailed,

}
