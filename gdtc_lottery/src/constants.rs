pub static TOKEN_PROGRAM_BYTES: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
pub static NFT_TOKEN_PROGRAM_BYTES: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
pub const MAX_PARTICIPANTS_PER_ROUND: u64 = 50; // 每轮最多 50 个 LP
pub const LP_PER_PARTICIPATION: u64 = 1 * 1000_000_000; // 每笔参与质押 1 个 LP
pub const PRIZE_POOL_AMOUNT: u64 = 10; // 每轮抽奖奖金池为 10 GDTC

pub static LOTTERY_SEED: &[u8] = b"lottery_instance";
pub static LOTTERY_ROUND_SEED: &[u8] = b"lottery_round_instance";
pub static LPTOKEN_SEED: &[u8] = b"lp_token";
pub static STAKING_SEED: &[u8] = b"staking_instance";
pub static USER_SEED: &[u8] = b"user_deposit";

// pub static Stake_CA: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"; //质押合约
