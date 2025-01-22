use anchor_lang::prelude::*;
pub mod error;
pub mod initialize_lottery;
pub mod initialize_lottery_round;
pub mod initialize_user;
pub mod participate;
pub mod claim_reward;
pub mod claim_lp;
pub mod admin_claim_reward;


#[account]
pub struct LotteryState {
    pub authority: Pubkey, // 管理员账户
    pub fund_foundation: Pubkey,    // 基金会地址
    pub reward_token_mint: Pubkey,  // 奖励代币 Mint 地址
    pub staking_token_mint: Pubkey, // 质押代币 Mint 地址
    pub lottery_number: u64,        //当前彩票轮次
}

#[account]
pub struct LotteryRound {
    pub round_number: u64,           // 当前抽奖的期数
    pub max_hash_values: u128, // 固定长度的元组数组，每个元素是参与者的 Pubkey 和去掉字母后的哈希值
    pub winner: Pubkey,        // 当前轮次的中奖者地址
    pub total_lp: u64,         // 当前轮次的 LP 总数
    pub round_start_time: i64, // 抽奖轮次开始时间 (UNIX 时间戳)
    pub round_end_time: Option<i64>, // 抽奖轮次结束时间 (UNIX 时间戳，可选)
    pub is_active: bool,       // 当前轮次是否活跃
    pub is_unstake: bool,      //是否已解除质押
    pub unclaim_lp_number: u64, //未领取的lp数量，初始值应为50
    pub reward_claimed: bool,  // 用户是否已经领取奖励
}

// #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
// pub struct Participant {
//     pub pubkey: Pubkey,       // 获胜者公钥
//     pub hash_value: u128,     // 获胜者的哈希值 去除子母后的数值
//     pub reward_claimed: bool, // 是否已解除质押
// }

#[account]
pub struct UserLotteryState {
    pub user_address: Pubkey,                         // 用户的公钥
    pub participated_rounds: [ParticipatedRound; 30], // 用户参与的轮次信息
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct ParticipatedRound {
    pub round_number: u64, // 用户参与的抽奖轮次
    pub is_unstaked: bool, // 用户是否取消了质押
    pub is_exist: bool,    //是否参与本轮彩票
}
