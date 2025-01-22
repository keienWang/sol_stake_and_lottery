use super::{LotteryRound, LotteryState};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction};

use gdtc_stake::structures::{StakingInstance,User,Staked};
use gdtc_stake::program::GdtcStaking;
use gdtc_stake::cpi::accounts::InitializeUser;
use anchor_spl::token::{Mint, Token, TokenAccount};

// 需要的账户结构
#[derive(Accounts)]
#[instruction(round_number:u64)]
pub struct InitializeLotteryRound<'info> {
    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(
    init, 
    payer = authority, 
    space = 8+core::mem::size_of::<LotteryRound>(),
    seeds = [crate::LOTTERY_ROUND_SEED.as_ref(),&round_number.to_le_bytes()], // 动态轮次号,
    bump
    )]
    pub lottery_round: Account<'info, LotteryRound>,
    #[account(mut)]
    pub authority: Signer<'info>, // 账户的签名者，通常是管理员

    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(
        mut,
        seeds = [crate::LPTOKEN_SEED.as_ref(),&round_number.to_le_bytes()], 
        bump,
    )]
    pub pda_account: AccountInfo<'info>, //合约pda账户 seed中加入了轮次，每轮的pda不同，可以质押超过10笔
    pub system_program: Program<'info, System>,

    #[account(mut)]
    pub staking_instance: Account<'info, StakingInstance>, // 质押合约的状态账户
     /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(
        // init, 
        // payer = authority, 
        // space = 8 + core::mem::size_of::<User>()+20 * core::mem::size_of::<Staked>(),
        // seeds = [
        //     crate::USER_SEED.as_ref(),
        //     staking_instance.key().as_ref(),
        //     pda_account.key().as_ref()
        // ],
        // bump
        mut
    )]
    pub user_instance: UncheckedAccount<'info>, //彩票合约指定期数的用户实例
    pub user_superior_token_account: Account<'info, TokenAccount>,
    pub staking_program: Program<'info, GdtcStaking>, // 质押合约
}

impl<'info> InitializeLotteryRound<'info> {
    pub fn process(&mut self, round_number: u64,bump_seed:u8) -> Result<()> {


        let transfer_sol_ix = system_instruction::transfer(
            &self.authority.key(), // 付款人（通常是用户）
            &self.pda_account.key(), // 接收者（PDA）
            5957760
        );
        
        invoke(
            &transfer_sol_ix,
            &[
                self.authority.to_account_info(),
                self.pda_account.to_account_info(),
                self.system_program.to_account_info(),
            ]
        )?;
        // 使用动态传入的 round_number
        let lottery_round = &mut self.lottery_round;

        // 设置 LotteryRound 账户的信息
        lottery_round.round_number = round_number;
        lottery_round.max_hash_values = 0;
        lottery_round.winner = Pubkey::default();
        lottery_round.total_lp = 0;
        lottery_round.round_start_time = Clock::get()?.unix_timestamp; // 当前时间戳作为开始时间
        lottery_round.round_end_time = None;
        lottery_round.is_active = true;
        lottery_round.is_unstake = true;
        lottery_round.unclaim_lp_number = 50; // 初始化未领取的 LP 数量为 50
        lottery_round.reward_claimed = false;

        
        if self.user_superior_token_account.owner != lottery_round.key(){
            return Err(ErrorCode::InvalidSuperiorTokenAccount.into());  // 如果上级 Token 账户无效，返回错误
        }

        if self.user_superior_token_account.mint != self.staking_instance.reward_token_mint
        {
           return Err(ErrorCode::InvalidTokenMint.into());
       }
        
        //注册本轮质押合约中的账户
        let signer_seeds: &[&[&[u8]]] = &[&[crate::LPTOKEN_SEED.as_ref(),&round_number.to_le_bytes(), &[bump_seed]]];

        let cpi_program = self.staking_program.to_account_info();

        // Passing the necessary account(s) to the `BobAddOp` account struct in Bob program
        let cpi_account = InitializeUser {
            // bob_data_account: self.bob_data_account.to_account_info(),
            authority:self.pda_account.to_account_info(),
            staking_instance:self.staking_instance.to_account_info(),
            user_instance:self.user_instance.to_account_info(),
            user_superior_token_account:self.user_superior_token_account.to_account_info(),
            system_program:self.system_program.to_account_info(),
        };

        // Creates a `CpiContext` object using the new method
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_account,signer_seeds);

        let res = gdtc_stake::cpi::initialize_user(cpi_ctx);

        // return an error if the CPI failed
        if res.is_ok() {
            return Ok(());
        } else {
            return Err(ErrorCode::CPIToStakeFailed.into())
        }
        return Ok(());
    }
}

#[error_code]
pub enum ErrorCode {
    
    #[msg("CPI call to staking program failed.")]
    CPIToStakeFailed,

    #[msg("The user superior token account is invalid.")]
    InvalidSuperiorTokenAccount,

    #[msg("Invalid token mint address.")]
    InvalidTokenMint,

}
