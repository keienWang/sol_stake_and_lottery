use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("The PDA account does not match.")]
    PdaAccountIsNotMatch,

    #[msg("Invalid token mint address.")]
    InvalidTokenMint,

    #[msg("Invalid account owner.")]
    InvalidAccountOwner,

    #[msg("The expected PDA account for the LP token does not match.")]
    InvalidLPTokensPdaOwner,

    #[msg("The lottery round number does not match the lottery state number.")]
    LotteryRoundNumberMismatch,

    #[msg("User has already participated in this lottery round.")]
    AlreadyParticipated,
}
