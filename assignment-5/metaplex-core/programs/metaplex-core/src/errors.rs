use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid reward point value. Must be between 0 and 10000.")]
    InvalidRewardPoint,
    #[msg("Invalid freeze period. Must be a positive integer.")]
    InvalidFreezePeriod,
    #[msg("Collection account is not owned by the program.")]
    InvalidCollectionAccount,
    #[msg("Update authority account is not owned by the program.")]
    InvalidUpdateAuthority,
    #[msg("Asset account is not owned by the program.")]
    InvalidOwner,
    #[msg("The asset is already staked.")]
    AlreadyStaked,
    #[msg("The asset is not currently staked.")]
    AssetNotStaked,
}
