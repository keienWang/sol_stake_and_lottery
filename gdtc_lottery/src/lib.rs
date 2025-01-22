use anchor_lang::prelude::*;
pub mod constants;
pub mod structures;

use constants::*;

use structures::{
    error::ErrorCode, initialize_lottery::*, initialize_lottery_round::*, initialize_user::*,
    participate::{*,Participate}, claim_reward::{*,ClaimReward},claim_lp::{*,ClaimLP},admin_claim_reward::{*,AdminClaimReward}
};

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("LUnrZKoeBMjvNWJZJmdZ7oEMq8R8PjVveavrGnTwCxc");

#[program]
pub mod gdtc_lottery {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let program_id = ctx.program_id; // 获取当前合约的程序ID
                                         // 计算 lottery_instance 的派生地址
        let (expected_pda_address, _) =
            Pubkey::find_program_address(&[crate::LOTTERY_SEED.as_ref()], program_id);

        if expected_pda_address != ctx.accounts.lottery_state.key() {
            return Err(ErrorCode::PdaAccountIsNotMatch.into());
        }
        ctx.accounts.process()
    }

    pub fn initialize_lottery_round(
        ctx: Context<InitializeLotteryRound>,
        round_number: u64, // 轮次号
    ) -> Result<()> {
        let program_id = ctx.program_id; // 获取当前合约的程序ID // 计算 lottery_instance 的派生地址
        let (expected_pda_address, _) = Pubkey::find_program_address(
            &[
                crate::LOTTERY_ROUND_SEED.as_ref(),
                &round_number.to_le_bytes(),
            ],
            program_id,
        );

        if expected_pda_address != ctx.accounts.lottery_round.key() {
            return Err(ErrorCode::PdaAccountIsNotMatch.into());
        }
       
        

        let bump_seed = ctx.bumps.pda_account;
        ctx.accounts.process(round_number,bump_seed)
    }

    pub fn initialize_user_lottery_state(ctx: Context<InitializeUserLotteryState>) -> Result<()> {
        let program_id = ctx.program_id; // 获取当前合约的程序ID
                                         // 计算 user_lottery_state 的派生地址
        let (expected_pda_address, _) =
            Pubkey::find_program_address(&[ctx.accounts.authority.key().as_ref()], program_id);

        if expected_pda_address != ctx.accounts.user_lottery_state.key() {
            return Err(ErrorCode::PdaAccountIsNotMatch.into());
        }
        ctx.accounts.process() // 调用初始化函数
    }

    pub fn participate(
        ctx: Context<Participate>,
        round_number: u64,
        user_index: u64,
    ) -> Result<()> {
        let program_id = ctx.program_id; // 获取当前合约的程序ID
        let (expected_lottery_state_pda_address, _) =
            Pubkey::find_program_address(&[crate::LOTTERY_SEED.as_ref()], program_id);

        let (expected_lottery_round_pda_address, _) = Pubkey::find_program_address(
            &[
                crate::LOTTERY_ROUND_SEED.as_ref(),
                &round_number.to_le_bytes(),
            ],
            program_id,
        );

        let (expected_user_lottery_state_pda_address, _) =
            Pubkey::find_program_address(&[ctx.accounts.authority.key().as_ref()], program_id);

        if expected_user_lottery_state_pda_address != ctx.accounts.user_lottery_state.key()
            && expected_lottery_round_pda_address != ctx.accounts.lottery_round.key()
            && expected_lottery_state_pda_address != ctx.accounts.lottery_state.key()
        {
            return Err(ErrorCode::PdaAccountIsNotMatch.into());
        }

        // 验证 gdtc_lp_in_account 的 mint 地址
        if ctx.accounts.gdtc_lp_in_account.mint != ctx.accounts.lottery_state.staking_token_mint
            && ctx.accounts.user_lp_token_account.mint
                != ctx.accounts.lottery_state.staking_token_mint&& ctx.accounts.gdtc_stake_lp_in_account.mint != ctx.accounts.lottery_state.reward_token_mint
        {
            return Err(ErrorCode::InvalidTokenMint.into());
        }

        //验证lptoken的pda账户
        let (expected_pda_address, _) =
            Pubkey::find_program_address(&[crate::LPTOKEN_SEED.as_ref()], program_id);

        if expected_pda_address != ctx.accounts.gdtc_lp_in_account.owner.key() {
            return Err(ErrorCode::InvalidAccountOwner.into());
        }

        // 验证 user_lp_token_account 的所有者（是否与签名账户匹配）
        if ctx.accounts.user_lp_token_account.owner != ctx.accounts.authority.key() {
            return Err(ErrorCode::InvalidAccountOwner.into());
        }
        let bump_seed = ctx.bumps.pda_account;
        ctx.accounts.process(bump_seed,user_index,round_number)
    }

    pub fn claim_reward(ctx: Context<ClaimReward>,round_number: u64,
        user_index: u64) -> Result<()> {
        let program_id = ctx.program_id; // 获取当前合约的程序ID
        let (expected_lottery_state_pda_address, _) =
            Pubkey::find_program_address(&[crate::LOTTERY_SEED.as_ref()], program_id);

        let (expected_lottery_round_pda_address, _) = Pubkey::find_program_address(
            &[
                crate::LOTTERY_ROUND_SEED.as_ref(),
                &round_number.to_le_bytes(),
            ],
            program_id,
        );

        let (expected_user_lottery_state_pda_address, _) =
            Pubkey::find_program_address(&[ctx.accounts.authority.key().as_ref()], program_id);

        if expected_user_lottery_state_pda_address != ctx.accounts.user_lottery_state.key()
            && expected_lottery_round_pda_address != ctx.accounts.lottery_round.key()
            && expected_lottery_state_pda_address != ctx.accounts.lottery_state.key()
        {
            return Err(ErrorCode::PdaAccountIsNotMatch.into());
        }

        // 验证  mint 地址
        if ctx.accounts.gdtc_reward_out_account.mint != ctx.accounts.lottery_state.reward_token_mint
            && ctx.accounts.user_gdtc_token_account.mint
                != ctx.accounts.lottery_state.reward_token_mint 
        {
            return Err(ErrorCode::InvalidTokenMint.into());
        }
        // 验证 user_gdtc_token_account 的所有者（是否与签名账户匹配）
        if ctx.accounts.user_gdtc_token_account.owner != ctx.accounts.authority.key() {
            return Err(ErrorCode::InvalidAccountOwner.into());
        }

         //验证lptoken的pda账户
         let (expected_pda_address, _) =
         Pubkey::find_program_address(&[crate::LPTOKEN_SEED.as_ref()], program_id);

     if expected_pda_address != ctx.accounts.gdtc_reward_out_account.owner.key() {
         return Err(ErrorCode::InvalidAccountOwner.into());
     }
        let bump_seed = ctx.bumps.pda_account;
        ctx.accounts.process(bump_seed,user_index)
    }

    pub fn claim_lp(ctx: Context<ClaimLP>,round_number: u64,
        user_index: u64) -> Result<()> {


            let program_id = ctx.program_id; // 获取当前合约的程序ID

        let (expected_lottery_state_pda_address, _) =
            Pubkey::find_program_address(&[crate::LOTTERY_SEED.as_ref()], program_id);

        let (expected_lottery_round_pda_address, _) = Pubkey::find_program_address(
            &[
                crate::LOTTERY_ROUND_SEED.as_ref(),
                &round_number.to_le_bytes(),
            ],
            program_id,
        );

        let (expected_user_lottery_state_pda_address, _) =
            Pubkey::find_program_address(&[ctx.accounts.authority.key().as_ref()], program_id);

        if expected_user_lottery_state_pda_address != ctx.accounts.user_lottery_state.key()
            && expected_lottery_round_pda_address != ctx.accounts.lottery_round.key()
            && expected_lottery_state_pda_address != ctx.accounts.lottery_state.key()
        {
            return Err(ErrorCode::PdaAccountIsNotMatch.into());
        }

        // 验证  mint 地址
        if ctx.accounts.user_lp_token_account.mint != ctx.accounts.lottery_state.staking_token_mint
            && ctx.accounts.gdtc_lp_in_account.mint
                != ctx.accounts.lottery_state.staking_token_mint &&
                 ctx.accounts.gdtc_stake_lp_in_account.mint!=ctx.accounts.lottery_state.staking_token_mint
        {
            return Err(ErrorCode::InvalidTokenMint.into());
        }

        // 验证 user_gdtc_token_account 的所有者（是否与签名账户匹配）
        if ctx.accounts.user_lp_token_account.owner != ctx.accounts.authority.key() {
            return Err(ErrorCode::InvalidAccountOwner.into());
        }

         //验证lptoken的pda账户
         let (expected_pda_address, _) =
         Pubkey::find_program_address(&[crate::LPTOKEN_SEED.as_ref()], program_id);

     if expected_pda_address != ctx.accounts.gdtc_lp_in_account.owner.key() {
         return Err(ErrorCode::InvalidAccountOwner.into());
     }

        let bump_seed = ctx.bumps.pda_account;
        ctx.accounts.process(bump_seed,user_index,round_number)
    }



    pub fn admin_claim_reward(ctx: Context<AdminClaimReward>
        ) -> Result<()> {
        let program_id = ctx.program_id; // 获取当前合约的程序ID
        
        ctx.accounts.process()
    }
}

