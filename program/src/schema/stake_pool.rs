use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use num_enum::TryFromPrimitive;
use solana_program::{
  msg,
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack, Sealed},
  pubkey::Pubkey,
};

///
/// StakePool state
///
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, TryFromPrimitive)]
pub enum StakePoolState {
  Uninitialized,
  Initialized,
  Frozen,
}
impl Default for StakePoolState {
  fn default() -> Self {
    StakePoolState::Uninitialized
  }
}

//
// Define the data struct
//
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StakePool {
  pub owner: Pubkey,
  pub state: StakePoolState,
  pub genesis_timestamp: i64,

  pub total_shares: u64,
  pub mint_share: Pubkey,

  pub mint_token: Pubkey,
  pub treasury_token: Pubkey,

  pub reward: u64,          // units: SEN / (share * seconds)
  pub period: u64,          // seconds
  pub compensation: i128,   // units: SEN / share, with 1e18 precision
  pub treasury_sen: Pubkey, // SEN Account
}

///
/// Pool implementation
///
impl StakePool {
  // Is frozen
  pub fn is_frozen(&self) -> bool {
    self.state == StakePoolState::Frozen
  }
}

//
// Implement Sealed trait
//
impl Sealed for StakePool {}

//
// Implement IsInitialized trait
//
impl IsInitialized for StakePool {
  fn is_initialized(&self) -> bool {
    self.state != StakePoolState::Uninitialized
  }
}

//
// Implement Pack trait
//
impl Pack for StakePool {
  // Fixed length
  const LEN: usize = 209;
  // Unpack data from [u8] to the data struct
  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    msg!("Read stake pool data");
    let src = array_ref![src, 0, 209];
    let (
      owner,
      state,
      genesis_timestamp,
      total_shares,
      mint_share,
      mint_token,
      treasury_token,
      reward,
      period,
      compensation,
      treasury_sen,
    ) = array_refs![src, 32, 1, 8, 8, 32, 32, 32, 8, 8, 16, 32];
    Ok(StakePool {
      owner: Pubkey::new_from_array(*owner),
      state: StakePoolState::try_from_primitive(state[0])
        .or(Err(ProgramError::InvalidAccountData))?,
      genesis_timestamp: i64::from_le_bytes(*genesis_timestamp),

      total_shares: u64::from_le_bytes(*total_shares),
      mint_share: Pubkey::new_from_array(*mint_share),

      mint_token: Pubkey::new_from_array(*mint_token),
      treasury_token: Pubkey::new_from_array(*treasury_token),

      reward: u64::from_le_bytes(*reward),
      period: u64::from_le_bytes(*period),
      compensation: i128::from_le_bytes(*compensation),
      treasury_sen: Pubkey::new_from_array(*treasury_sen),
    })
  }
  // Pack data from the data struct to [u8]
  fn pack_into_slice(&self, dst: &mut [u8]) {
    msg!("Write stake pool data");
    let dst = array_mut_ref![dst, 0, 209];
    let (
      dst_owner,
      dst_state,
      dst_genesis_timestamp,
      dst_total_shares,
      dst_mint_share,
      dst_mint_token,
      dst_treasury_token,
      dst_reward,
      dst_period,
      dst_compensation,
      dst_treasury_sen,
    ) = mut_array_refs![dst, 32, 1, 8, 8, 32, 32, 32, 8, 8, 16, 32];
    let &StakePool {
      ref owner,
      state,
      genesis_timestamp,
      total_shares,
      ref mint_share,
      ref mint_token,
      ref treasury_token,
      reward,
      period,
      compensation,
      ref treasury_sen,
    } = self;
    dst_owner.copy_from_slice(owner.as_ref());
    *dst_state = [state as u8];
    *dst_genesis_timestamp = genesis_timestamp.to_le_bytes();
    *dst_total_shares = total_shares.to_le_bytes();
    dst_mint_share.copy_from_slice(mint_share.as_ref());
    dst_mint_token.copy_from_slice(mint_token.as_ref());
    dst_treasury_token.copy_from_slice(treasury_token.as_ref());
    *dst_reward = reward.to_le_bytes();
    *dst_period = period.to_le_bytes();
    *dst_compensation = compensation.to_le_bytes();
    dst_treasury_sen.copy_from_slice(treasury_sen.as_ref());
  }
}
