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
  pub mint_share: Pubkey,
  pub vault: Pubkey, // SEN Account

  pub total_token_locked: u64,
  pub mint_token: Pubkey,
  pub treasury_token: Pubkey,

  pub reward: u64,          // units: SEN / (share * time)
  pub debt: u64,            // units: SEN / share
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
  const LEN: usize = 217;
  // Unpack data from [u8] to the data struct
  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    msg!("Read stake pool data");
    let src = array_ref![src, 0, 217];
    let (
      owner,
      state,
      mint_share,
      vault,
      total_token_locked,
      mint_token,
      treasury_token,
      reward,
      debt,
      treasury_sen,
    ) = array_refs![src, 32, 1, 32, 32, 8, 32, 32, 8, 8, 32];
    Ok(StakePool {
      owner: Pubkey::new_from_array(*owner),
      state: StakePoolState::try_from_primitive(state[0])
        .or(Err(ProgramError::InvalidAccountData))?,

      mint_share: Pubkey::new_from_array(*mint_share),
      vault: Pubkey::new_from_array(*vault),

      total_token_locked: u64::from_le_bytes(*total_token_locked),
      mint_token: Pubkey::new_from_array(*mint_token),
      treasury_token: Pubkey::new_from_array(*treasury_token),

      reward: u64::from_le_bytes(*reward),
      debt: u64::from_le_bytes(*debt),
      treasury_sen: Pubkey::new_from_array(*treasury_sen),
    })
  }
  // Pack data from the data struct to [u8]
  fn pack_into_slice(&self, dst: &mut [u8]) {
    msg!("Write stake pool data");
    let dst = array_mut_ref![dst, 0, 217];
    let (
      dst_owner,
      dst_state,
      dst_mint_share,
      dst_vault,
      dst_total_token_locked,
      dst_mint_token,
      dst_treasury_token,
      dst_reward,
      dst_debt,
      dst_treasury_sen,
    ) = mut_array_refs![dst, 32, 1, 32, 32, 8, 32, 32, 8, 8, 32];
    let &StakePool {
      ref owner,
      state,
      ref mint_share,
      ref vault,
      total_token_locked,
      ref mint_token,
      ref treasury_token,
      reward,
      debt,
      ref treasury_sen,
    } = self;
    dst_owner.copy_from_slice(owner.as_ref());
    *dst_state = [state as u8];
    dst_mint_share.copy_from_slice(mint_share.as_ref());
    dst_vault.copy_from_slice(vault.as_ref());
    *dst_total_token_locked = total_token_locked.to_le_bytes();
    dst_mint_token.copy_from_slice(mint_token.as_ref());
    dst_treasury_token.copy_from_slice(treasury_token.as_ref());
    *dst_reward = reward.to_le_bytes();
    *dst_debt = debt.to_le_bytes();
    dst_treasury_sen.copy_from_slice(treasury_sen.as_ref());
  }
}
