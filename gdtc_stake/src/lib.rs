pub mod constants;
pub mod structures;
pub mod tools;

use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::token;
use constants::*;
use structures::{
    cancel_staking::*, claim_rewards::*, enter_staking::*, initialize_staking::*,
    initialize_user::*, Staked, StakingInstance, StakingPool, User,
};
use tools::{generate_release_timestamps, test_generate_release_timestamp};

declare_id!("H79TrubVu9ParAtDtuYqzKVZP3TR531sPxoDqaeA8KXK");

pub fn update_reward_pool(current_timestamp: u64, staking_instance: &mut StakingInstance) {
    // 遍历每个质押池
    for pool in staking_instance.pools.iter_mut() {
        // 如果没有份额，跳过此池
        if pool.total_shares == 0 {
            continue;
        }
        // 计算时间差（当前时间戳 - 上次奖励时间戳）
        let time_diff = current_timestamp
            .checked_sub(pool.last_reward_timestamp)
            .unwrap_or(0);

        // 如果时间差为 0，跳过此池
        if time_diff == 0 {
            continue;
        }

        // 计算池子的总奖励收入（奖励速率 * 时间差）
        let income = pool
            .reward_token_per_sec
            .checked_mul(time_diff)
            .unwrap_or(0);

        // 更新 `accumulated_reward_per_share`
        if pool.total_shares > 0 {
            // 每份奖励计算
            let reward_per_share = (income as u128)
                .checked_mul(COMPUTATION_DECIMALS as u128) // 精度调整
                .unwrap_or(0)
                .checked_div(pool.total_shares as u128) // 每份奖励
                .unwrap_or(0) as u64;

            // 累加每份奖励的累计值
            pool.accumulated_reward_per_share = pool
                .accumulated_reward_per_share
                .checked_add(reward_per_share)
                .unwrap_or(pool.accumulated_reward_per_share); // 防止溢出
        }

        // 更新最后奖励时间戳为当前时间戳
        pool.last_reward_timestamp = current_timestamp;
    }
}

pub fn store_pending_reward(
    staking_instance: &mut StakingInstance,
    user_instance: &mut User,
    staked_info_number: u64, // 修改为索引
) -> Result<()> {
    // 获取用户对应的质押信息
    let staked_info = &mut user_instance.staked_info[staked_info_number as usize];

    // 确保该质押池已被质押
    if !staked_info.is_staked {
        return Ok(()); // 如果该质押池没有质押，直接返回
    }

    // 获取质押类型对应的池子
    let stake_type = staked_info.stake_type as usize;

    // 检查 stake_type 是否为有效池子索引
    // if stake_type >= staking_instance.pools.len() {
    //     return Err(ErrorCode::InvalidStakeType.into()); // 自定义错误类型
    // }

    // 获取对应池子
    let pool = &staking_instance.pools[stake_type];

    // 计算用户在该池子的待领取奖励
    let pending_reward = (staked_info.deposited_amount as u128)
        .checked_mul(pool.accumulated_reward_per_share as u128)
        .and_then(|v| v.checked_div(COMPUTATION_DECIMALS as u128))
        .and_then(|v| v.checked_sub(staked_info.reward_debt as u128))
        .unwrap_or(0) as u64; // 最终将结果转换回 u64 类型，如果需要
                              // 如果待领取奖励为 0，直接返回
    if pending_reward == 0 {
        return Ok(());
    }

    // 更新该质押池的累计奖励
    staked_info.accumulated_reward = staked_info
        .accumulated_reward
        .checked_add(pending_reward)
        .unwrap_or(staked_info.accumulated_reward); // 防止溢出

    // 更新用户的 reward_debt 为最新的池子状态
    staked_info.reward_debt = (staked_info.deposited_amount as u128)
        .checked_mul(pool.accumulated_reward_per_share as u128)
        .and_then(|v| v.checked_div(COMPUTATION_DECIMALS as u128))
        .unwrap_or(staked_info.reward_debt as u128) as u64;
    Ok(())
}

pub fn update_reward_debt(
    staking_instance: &mut StakingInstance,
    user_instance: &mut User,
    staked_info_number: u64, // 用户质押池的索引
) {
    // 获取用户对应的质押信息
    let staked_info = &mut user_instance.staked_info[staked_info_number as usize];

    // 确保该质押池已被质押
    if !staked_info.is_staked {
        return; // 如果该质押池没有质押，直接返回
    }
    // 获取质押类型对应的池子
    let stake_type = staked_info.stake_type as usize;
    // 检查 stake_type 是否为有效池子索引
    if stake_type >= staking_instance.pools.len() {
        return; // 无效的池子索引，直接返回
    }

    // 获取对应池子
    let pool = &staking_instance.pools[stake_type];

    // msg!(
    //     "Hello world!!",
    //     pool.accumulated_reward_per_share,
    //     staked_info.deposited_amount
    // );
    // 更新该质押池的 reward_debt
    // msg!(
    //     "staked_info
    //     .deposited_amount",
    //     staked_info.deposited_amount
    // );
    // msg!("accumulated_reward_per_share", accumulated_reward_per_share);

    staked_info.reward_debt = (staked_info.deposited_amount as u128)
        .checked_mul(pool.accumulated_reward_per_share as u128)
        .and_then(|v| v.checked_div(COMPUTATION_DECIMALS as u128))
        .unwrap_or(0) as u64;
}

pub fn is_authorized(user: &Pubkey, authority: &Pubkey) -> bool {
    user == authority
}
pub fn can_unstake(staked: &Staked, current_timestamp: u64) -> bool {
    staked.is_staked && staked.stake_end_time <= current_timestamp
}

pub fn calculate_referral_reward(user: &User, amount: u64) -> u64 {
    // 计算推荐奖励，假设为10%
    let referral_reward = amount * 10 / 100;
    referral_reward
}

#[program]
pub mod gdtc_staking {
    use super::*;
    pub fn initialize_staking(
        ctx: Context<InitializeStaking>,
        reward_per_sec_3_months: u64,
        reward_per_sec_6_months: u64,
        reward_per_sec_12_months: u64,
        start_reward_timestamp: u64,
    ) -> Result<()> {
        let staking_instance = &mut ctx.accounts.staking_instance;

        // 设置基础字段
        staking_instance.authority = ctx.accounts.authority.key();
        staking_instance.reward_token_mint = ctx.accounts.reward_token_mint.key();
        staking_instance.staking_token_mint = ctx.accounts.staking_token_mint.key();
        staking_instance.lp_token_account = ctx.accounts.lp_token_account.key();

        let program_id = ctx.program_id; // 获取当前合约的程序ID
                                         // 计算 staking_instance 的派生地址
        let (expected_pda_address, bump_seed) =
            Pubkey::find_program_address(&[crate::LPTOKEN_SEED.as_ref()], program_id);

        if expected_pda_address != ctx.accounts.lp_token_account.owner.key() {
            return Err(ErrorCode::PdaAccountIsNotMatch.into());
        }

        // 初始化 3 个质押池
        staking_instance.pools = [
            StakingPool {
                stake_type: 0, // 3 个月
                reward_token_per_sec: reward_per_sec_3_months,
                accumulated_reward_per_share: 0,
                last_reward_timestamp: start_reward_timestamp,
                total_shares: 0,
            },
            StakingPool {
                stake_type: 1, // 6 个月
                reward_token_per_sec: reward_per_sec_6_months,
                accumulated_reward_per_share: 0,
                last_reward_timestamp: start_reward_timestamp,
                total_shares: 0,
            },
            StakingPool {
                stake_type: 2, // 12 个月
                reward_token_per_sec: reward_per_sec_12_months,
                accumulated_reward_per_share: 0,
                last_reward_timestamp: start_reward_timestamp,
                total_shares: 0,
            },
        ];
        Ok(())
    }

    pub fn initialize_user(ctx: Context<InitializeUser>) -> Result<()> {
        let user_instance = &mut ctx.accounts.user_instance;
        let staking_instance = &mut ctx.accounts.staking_instance;
        if staking_instance.reward_token_mint != ctx.accounts.user_superior_token_account.mint {
            return Err(ErrorCode::MintAccountIsNotMatch.into());
        }
        user_instance.user_address = ctx.accounts.authority.key();
        // 初始化 User 结构体的字段
        user_instance.total_deposited_amount = 0; // 初始化为 0，表示用户没有存入任何质押
        user_instance.user_superior_token_account = ctx.accounts.user_superior_token_account.key(); // 设置上级 Token 账户地址
        user_instance.isinit = true; // 标记为已初始化

        // 初始化 staked_info 数组，所有的质押池信息都设为默认值
        for staked in user_instance.staked_info.iter_mut() {
            staked.deposited_amount = 0; // 每个质押池的存入金额初始化为 0
            staked.reward_debt = 0; // 奖励债务初始化为 0
            staked.accumulated_reward = 0; // 累计奖励初始化为 0
            staked.is_staked = false; // 初始时没有质押
            staked.stake_type = 0; // 默认的质押类型为 0 (3个月质押)
            staked.stake_start_time = 0; // 初始质押开始时间为 0
            staked.stake_end_time = 0; // 初始质押结束时间为 0
            staked.receivedReward = 0; //初始化已领取收益
            staked.can_cancel_stake = false;
        }

        Ok(())
    }

    // 0 = 3m,1= 6m,2= 12m;
    pub fn enter_staking(
        ctx: Context<EnterStaking>,
        lp_staking_number: u64, // 用户要质押的 LP Token 数量
        stake_type: u64,        // 用户选择的质押池类型
        staked_info_index: u64, // 用户选择的 staked_info 索引
    ) -> Result<()> {
        // 获取账户实例
        let staking_instance = &mut ctx.accounts.staking_instance;
        let user_instance = &mut ctx.accounts.user_instance;
        let user_lp_token_account = &ctx.accounts.user_lp_token_account;
        let gdtc_lp_in_account = &ctx.accounts.gdtc_lp_in_account;

        let clock = Clock::get().expect("Failed to get clock");

        let program_id = ctx.program_id; // 获取当前合约的程序ID
                                         // 计算 staking_instance 的派生地址
        let (expected_staking_address, bump_seed) =
            Pubkey::find_program_address(&[crate::STAKING_SEED.as_ref()], program_id);

        // 确保 staking_instance 是由合约程序派生的
        if staking_instance.key() != expected_staking_address {
            return Err(ErrorCode::InvalidStakingInstance.into());
        }
        //用户账户验证
        let (expected_user_address, bump_seed) = Pubkey::find_program_address(
            &[
                crate::USER_SEED.as_ref(),
                staking_instance.key().as_ref(),
                ctx.accounts.authority.key().as_ref(),
            ],
            program_id,
        );
        msg!(
            "expected_user_address is: {},user_instance: {}",
            expected_user_address.key(),
            user_instance.key()
        );
        // 确保 staking_instance 是由合约程序派生的
        if user_instance.key() != expected_user_address {
            return Err(ErrorCode::InvalidUserInstance.into());
        }

        // 检查用户 LP Token 账户的 Mint 是否与质押池的 Mint 匹配
        if staking_instance.staking_token_mint != user_lp_token_account.mint {
            return Err(ErrorCode::MintAccountIsNotMatch.into());
        }
        if staking_instance.lp_token_account != gdtc_lp_in_account.key() {
            return Err(ErrorCode::MintAccountIsNotMatch.into());
        }
        if user_instance.user_address != ctx.accounts.authority.key() {
            return Err(ErrorCode::UserAccountIsNotMatch.into());
        }
        // 检查用户 LP Token 账户余额是否足够
        if user_lp_token_account.amount < lp_staking_number {
            return Err(ErrorCode::TokenAccountBalanceInsufficient.into());
        }
        if staked_info_index > 9 {
            return Err(ErrorCode::InvalidStakedInfoIndex.into());
        }

        if stake_type > 2 {
            return Err(ErrorCode::InvalidStakeType.into());
        }
        // 获取当前时间戳并计算质押结束时间
        let current_timestamp = clock.unix_timestamp as u64;

        msg!("staking time is :{}", current_timestamp);
        let is_end = match stake_type {
            0 => {
                if current_timestamp > 2358810461 {
                    true
                } else {
                    false
                }
            }
            1 => {
                if current_timestamp > 2350861661 {
                    true
                } else {
                    false
                }
            }
            2 => {
                if current_timestamp > 2335136861 {
                    true
                } else {
                    false
                }
            }
            _ => true, // 对于无效的 stake_type，直接认为已经结束
        };

        if is_end {
            return Err(ErrorCode::StakingEnded.into());
        }

        // user_instance.total_deposited_amount = user_instance
        //     .total_deposited_amount
        //     .checked_add(lp_staking_number)
        //     .ok_or(ErrorCode::Overflow)?;

        // 获取质押信息
        let index = staked_info_index as usize;
        // 如果已经质押，报错
        if user_instance.staked_info[index].is_staked {
            return Err(ErrorCode::UserAlreadyStaked.into());
        }
        // 验证用户选择的质押池类型是否有效
        if stake_type >= staking_instance.pools.len() as u64 {
            return Err(ErrorCode::InvalidStakeType.into());
        }

        let stake_end_time = generate_release_timestamps(current_timestamp, stake_type);

        // 更新用户账户
        user_instance.total_deposited_amount = user_instance
            .total_deposited_amount
            .checked_add(lp_staking_number)
            .ok_or(ErrorCode::Overflow)?;

        let staked_info = &mut user_instance.staked_info[index];

        staked_info.deposited_amount = staked_info
            .deposited_amount
            .checked_add(lp_staking_number)
            .ok_or(ErrorCode::Overflow)?;
        staked_info.stake_type = stake_type;
        staked_info.is_staked = true;
        staked_info.stake_start_time = current_timestamp;
        staked_info.stake_end_time = stake_end_time;

        // 更新质押池的总份额
        let pool = &mut staking_instance.pools[stake_type as usize];
        pool.total_shares = pool
            .total_shares
            .checked_add(lp_staking_number)
            .ok_or(ErrorCode::Overflow)?;

        // 更新奖励池
        update_reward_pool(current_timestamp, staking_instance);

        // 更新用户奖励债务
        update_reward_debt(staking_instance, user_instance, staked_info_index);

        // 转移 LP Token 到合约的 Vault
        token::transfer(
            ctx.accounts.into_transfer_to_vault_context(),
            lp_staking_number,
        )?;

        Ok(())
    }

    pub fn cancel_staking(ctx: Context<CancelStaking>, staked_info_index: u64) -> Result<()> {
        // 获取相关账户
        let staking_instance = &mut ctx.accounts.staking_instance;
        let user_instance = &mut ctx.accounts.user_instance;
        let user_lp_token_account = &mut ctx.accounts.user_lp_token_account;
        let gdtc_lp_in_account = &ctx.accounts.gdtc_lp_in_account;

        let program_id = ctx.program_id; // 获取当前合约的程序ID

        let (expected_staking_address, bump_seed) =
            Pubkey::find_program_address(&[crate::STAKING_SEED.as_ref()], program_id);

        // 确保 staking_instance 是由合约程序派生的
        if staking_instance.key() != expected_staking_address {
            return Err(ErrorCode::InvalidStakingInstance.into());
        }
        //用户账户验证
        let (expected_user_address, bump_seed) = Pubkey::find_program_address(
            &[
                crate::USER_SEED.as_ref(),
                staking_instance.key().as_ref(),
                ctx.accounts.authority.key().as_ref(),
            ],
            program_id,
        );
        msg!(
            "expected_user_address is: {},user_instance: {}",
            expected_user_address.key(),
            user_instance.key()
        );
        // 确保 staking_instance 是由合约程序派生的
        if user_instance.key() != expected_user_address {
            return Err(ErrorCode::InvalidUserInstance.into());
        }

        if staking_instance.staking_token_mint != user_lp_token_account.mint {
            return Err(ErrorCode::MintAccountIsNotMatch.into());
        }
        if staking_instance.lp_token_account != gdtc_lp_in_account.key() {
            return Err(ErrorCode::MintAccountIsNotMatch.into());
        }
        if user_instance.user_address != ctx.accounts.authority.key() {
            return Err(ErrorCode::UserAccountIsNotMatch.into());
        }

        let index = staked_info_index as usize;

        let amount = user_instance.staked_info[index].deposited_amount;

        // 检查用户是否有质押
        if !user_instance.staked_info[index].is_staked {
            return Err(ErrorCode::NoStakingToCancel.into());
        }

        // 获取当前时间戳
        let clock = Clock::get().map_err(|_| ErrorCode::ClockUnavailable)?;
        let current_timestamp = clock.unix_timestamp as u64;

        // 检查质押是否到期
        if current_timestamp < user_instance.staked_info[index].stake_end_time {
            return Err(ErrorCode::StakingNotMatured.into());
        }
        if !user_instance.staked_info[index].can_cancel_stake {
            return Err(ErrorCode::NeedCliamRewards.into());
        }

        // 更新奖励池并计算用户的奖励
        update_reward_pool(current_timestamp, staking_instance);

        // 存储用户的待领取奖励
        store_pending_reward(staking_instance, user_instance, staked_info_index)?;

        // 获取 PDA 签名者
        let bump_seed = ctx.bumps.pda_account;
        let signer_seeds: &[&[&[u8]]] = &[&[crate::LPTOKEN_SEED.as_ref(), &[bump_seed]]];

        // 生成从 GDTC 托管账户到用户 LP Token 账户的转账指令
        let transfer_instruction = spl_token::instruction::transfer(
            &ctx.accounts.token_program.key(),
            &ctx.accounts.gdtc_lp_in_account.key(),
            &ctx.accounts.user_lp_token_account.key(),
            &ctx.accounts.pda_account.key(),
            &[],
            amount,
        )?;

        // 执行带签名的 CPI 调用
        invoke_signed(
            &transfer_instruction,
            &[
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.gdtc_lp_in_account.to_account_info(),
                ctx.accounts.user_lp_token_account.to_account_info(),
                ctx.accounts.pda_account.to_account_info(),
            ],
            signer_seeds,
        )?;

        // // 更新质押池的总份额
        let pool =
            &mut staking_instance.pools[user_instance.staked_info[index].stake_type as usize];

        pool.total_shares = pool
            .total_shares
            .checked_sub(amount)
            .ok_or(ErrorCode::Underflow)?;

        // 更新奖励债务
        update_reward_debt(staking_instance, user_instance, staked_info_index);

        // 获取用户对应的质押信息
        let staked_info = &mut user_instance.staked_info[index];
        // 重置用户的质押状态
        staked_info.deposited_amount = 0;
        staked_info.accumulated_reward = 0;
        staked_info.is_staked = false; // 标记用户未质押
        staked_info.stake_type = 0;
        staked_info.reward_debt = 0; // 重置奖励债务
        staked_info.stake_start_time = 0; // 重置质押开始时间
        staked_info.stake_end_time = 0; // 重置质押结束时间
        staked_info.receivedReward = 0;
        user_instance.staked_info[index].can_cancel_stake = false;

        Ok(())
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>, staked_info_index: u64) -> Result<()> {
        // 获取账户实例
        let staking_instance = &mut ctx.accounts.staking_instance;
        let user_instance = &mut ctx.accounts.user_instance;
        let super_instance = &ctx.accounts.super_instance;

        // let user_gdtc_token_account = &mut ctx.accounts.user_gdtc_token_account;
        let gdtc_reward_out_account = &ctx.accounts.gdtc_reward_out_account;
        let user_super_gdtc_token_account = &mut ctx.accounts.user_super_gdtc_token_account;

        let program_id = ctx.program_id; // 获取当前合约的程序ID
                                         // 计算 staking_instance 的派生地址
        let (expected_staking_address, bump_seed) =
            Pubkey::find_program_address(&[crate::STAKING_SEED.as_ref()], program_id);

        // 确保 staking_instance 是由合约程序派生的
        if staking_instance.key() != expected_staking_address {
            return Err(ErrorCode::InvalidStakingInstance.into());
        }

        let (expected_pda_address, bump_seed) =
            Pubkey::find_program_address(&[crate::LPTOKEN_SEED.as_ref()], program_id);

        if expected_pda_address != gdtc_reward_out_account.owner.key() {
            return Err(ErrorCode::PdaAccountIsNotMatch.into());
        }

        //用户账户验证
        let (expected_user_address, bump_seed) = Pubkey::find_program_address(
            &[
                crate::USER_SEED.as_ref(),
                staking_instance.key().as_ref(),
                user_instance.user_address.key().as_ref(),
            ],
            program_id,
        );
        msg!(
            "expected_user_address is: {},user_instance: {}",
            expected_user_address.key(),
            user_instance.key()
        );
        // 确保 staking_instance 是由合约程序派生的
        if user_instance.key() != expected_user_address {
            return Err(ErrorCode::InvalidUserInstance.into());
        }

        //用户上级账户验证
        let (expected_user_superior_address, bump_seed) = Pubkey::find_program_address(
            &[
                crate::USER_SEED.as_ref(),
                staking_instance.key().as_ref(),
                user_super_gdtc_token_account.owner.key().as_ref(),
            ],
            program_id,
        );
        msg!(
            "expected_user_superior_address is: {},user_instance: {}",
            expected_user_superior_address.key(),
            super_instance.key()
        );
        // 确保 staking_instance 是由合约程序派生的
        if super_instance.key() != expected_user_superior_address {
            return Err(ErrorCode::InvalidUserInstance.into());
        }

        if staking_instance.reward_token_mint != gdtc_reward_out_account.mint {
            return Err(ErrorCode::MintAccountIsNotMatch.into());
        }

        // 获取当前时间戳
        let clock = Clock::get().map_err(|_| ErrorCode::ClockUnavailable)?;
        let current_timestamp = clock.unix_timestamp as u64;
        let index = staked_info_index as usize;
        if user_instance.user_address != ctx.accounts.user_gdtc_token_account.owner.key() {
            return Err(ErrorCode::UserAccountIsNotMatch.into());
        }
        if super_instance.user_address != user_super_gdtc_token_account.owner.key() {
            return Err(ErrorCode::UserAccountIsNotMatch.into());
        }

        // 检查用户是否有质押
        if !user_instance.staked_info[index].is_staked {
            msg!(
                "index: {},staked:{}",
                staked_info_index,
                user_instance.staked_info[index].is_staked
            );
            return Err(ErrorCode::NoStakingToClaimRewards.into());
            // return Ok(());
        }
        msg!(
            "index{},staked:{}",
            index,
            user_instance.staked_info[index].is_staked
        );
        if user_instance.user_superior_token_account != user_super_gdtc_token_account.key() {
            return Err(ErrorCode::MintAccountIsNotMatch.into());
        }

        // 更新奖励池并计算用户的奖励
        update_reward_pool(current_timestamp, staking_instance);

        store_pending_reward(staking_instance, user_instance, staked_info_index)?;

        // 更新用户的奖励债务
        // update_reward_debt(staking_instance, user_instance, staked_info_index);

        // 计算用户的奖励
        let mut accumulated_reward = user_instance.staked_info[index].accumulated_reward;
        if accumulated_reward == 0 {
            return Err(ErrorCode::NoRewardsToClaim.into());
        }

        // 检查奖励账户余额是否足够
        if gdtc_reward_out_account.amount < accumulated_reward {
            if current_timestamp >= user_instance.staked_info[index].stake_end_time
                && user_instance.user_address == ctx.accounts.authority.key()
            {
                if user_instance.staked_info[index].can_cancel_stake != true {
                    user_instance.staked_info[index].can_cancel_stake = true;
                    user_instance.total_deposited_amount = user_instance
                        .total_deposited_amount
                        .checked_sub(user_instance.staked_info[index].deposited_amount)
                        .ok_or(ErrorCode::Overflow)?;
                }
            }
            return Ok(());
        }

        let bump_seed = ctx.bumps.pda_account;
        let signer_seeds: &[&[&[u8]]] = &[&[crate::LPTOKEN_SEED.as_ref(), &[bump_seed]]];

        if super_instance.total_deposited_amount > 2000000000 {
            let transfer_instruction = spl_token::instruction::transfer(
                &ctx.accounts.token_program.key(),
                &ctx.accounts.gdtc_reward_out_account.key(),
                &ctx.accounts.user_super_gdtc_token_account.key(),
                &ctx.accounts.pda_account.key(),
                &[],
                accumulated_reward / 10,
            )?;

            // 执行带签名的 CPI 调用
            invoke_signed(
                &transfer_instruction,
                &[
                    ctx.accounts.token_program.to_account_info(),
                    ctx.accounts.gdtc_reward_out_account.to_account_info(),
                    ctx.accounts.user_super_gdtc_token_account.to_account_info(),
                    ctx.accounts.pda_account.to_account_info(),
                ],
                signer_seeds,
            )?;
            //取消上级百分之十从挖矿者奖励中拿出
            // accumulated_reward = accumulated_reward - (accumulated_reward / 10);
        }

        // 生成从 GDTC 托管账户到用户 LP Token 账户的转账指令
        let transfer_instruction = spl_token::instruction::transfer(
            &ctx.accounts.token_program.key(),
            &ctx.accounts.gdtc_reward_out_account.key(),
            &ctx.accounts.user_gdtc_token_account.key(),
            &ctx.accounts.pda_account.key(),
            &[],
            accumulated_reward,
        )?;

        // 执行带签名的 CPI 调用
        invoke_signed(
            &transfer_instruction,
            &[
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.gdtc_reward_out_account.to_account_info(),
                ctx.accounts.user_gdtc_token_account.to_account_info(),
                ctx.accounts.pda_account.to_account_info(),
            ],
            signer_seeds,
        )?;

        if current_timestamp >= user_instance.staked_info[index].stake_end_time {
            if user_instance.staked_info[index].can_cancel_stake == true {
                msg!(
                    "can_cancel_stake,{},index : {}",
                    user_instance.staked_info[index].can_cancel_stake,
                    index
                );
                return Err(ErrorCode::NoRewardsToClaim.into());
            }
            user_instance.staked_info[index].can_cancel_stake = true;
            user_instance.total_deposited_amount = user_instance
                .total_deposited_amount
                .checked_sub(user_instance.staked_info[index].deposited_amount)
                .ok_or(ErrorCode::Overflow)?;
        }

        // 重置用户累计奖励
        user_instance.staked_info[index].accumulated_reward = 0;

        user_instance.staked_info[index].receivedReward = user_instance.staked_info[index]
            .receivedReward
            .checked_add(accumulated_reward)
            .ok_or(ErrorCode::Overflow)?;
        Ok(())
    }

}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid stake type provided. The stake type must correspond to an existing pool.")]
    InvalidStakeType,

    #[msg("Invalid stake staked info index.")]
    InvalidStakedInfoIndex,

    #[msg("Insufficient token account balance.")]
    TokenAccountBalanceInsufficient,

    #[msg("Failed to fetch system clock.")]
    ClockUnavailable,

    #[msg("User token account mint does not match staking token mint.")]
    MintAccountIsNotMatch,

    #[msg("Arithmetic overflow occurred.")]
    Overflow,

    #[msg("Arithmetic underflow occurred.")]
    Underflow,

    #[msg("User has already staked and cannot stake again.")]
    UserAlreadyStaked,

    #[msg("User has no staking to cancel.")]
    NoStakingToCancel,

    #[msg("Staking period has not matured yet.")]
    StakingNotMatured,

    #[msg("No rewards available to claim.")]
    NoRewardsToClaim,

    #[msg("Insufficient reward account balance.")]
    InsufficientRewardBalance, // 奖励账户余额不足

    #[msg("No Staking available to claim.")]
    NoStakingToClaimRewards,

    #[msg("UserSuperiorTokenAccount  does not match.")]
    UserSuperiorTokenAccountIsNotMatch,

    #[msg("User address  does not match.")]
    UserAccountIsNotMatch,

    #[msg("User need cliam rewards.")]
    NeedCliamRewards,

    #[msg("The provided staking instance is not a valid staking instance for this contract.")]
    InvalidStakingInstance,

    #[msg("The staking has ended for this instance.")]
    StakingEnded,

    #[msg("The provided user instance is not a valid user instance for this contract.")]
    InvalidUserInstance,

    #[msg("Pda address  does not match.")]
    PdaAccountIsNotMatch,
}
