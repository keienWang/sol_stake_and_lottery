use anchor_lang::prelude::*;
pub mod cancel_staking;
pub mod claim_rewards;
pub mod enter_staking;
pub mod initialize_staking;
pub mod initialize_user;

// staking structures
#[account]
pub struct StakingInstance {
    pub authority: Pubkey,          // 管理员账户
    pub reward_token_mint: Pubkey,  // 奖励代币 Mint 地址
    pub staking_token_mint: Pubkey, // 质押代币 Mint 地址
    pub pools: [StakingPool; 3],    // 固定3个质押池
    pub lp_token_account: Pubkey,   //合约接受lp的合约地址
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct StakingPool {
    pub stake_type: u64, // 0 代表3个月，1 代表6个月，2代表12个月
    pub reward_token_per_sec: u64, // 每秒奖励代币数量
    pub accumulated_reward_per_share: u64, // 累计奖励分摊
    pub last_reward_timestamp: u64, // 上次更新奖励的时间戳
    pub total_shares: u64, // 该池中质押的总份额
}

#[account]
pub struct User {
    //这个字段必须第一位
    pub total_deposited_amount: u64, // 用户总存入的质押金额
    pub user_superior_token_account: Pubkey, // 用户的上级 Token 账户
    pub staked_info: [Staked; 10],   // 固定10个质押池
    pub isinit: bool,
    pub user_address: Pubkey,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Staked {
    pub deposited_amount: u64,   // 用户总存入的质押金额
    pub reward_debt: u64,        // 用户奖励债务
    pub accumulated_reward: u64, // 用户累计获得的奖励
    pub is_staked: bool,         // 用户是否已质押
    pub stake_type: u64,         // 质押类型
    pub stake_start_time: u64,   // 质押开始时间（Unix 时间戳）
    pub stake_end_time: u64,     // 质押结束时间（Unix 时间戳）
    pub receivedReward: u64,     //已领取收益
    pub can_cancel_stake: bool,  //是否可以解除质押
}
