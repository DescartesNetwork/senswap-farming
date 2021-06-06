use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
  msg,
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack, Sealed},
  pubkey::Pubkey,
};

//
// Define the data struct
//
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Debt {
  pub stake_pool: Pubkey,
  pub owner: Pubkey,
  pub account: Pubkey,
  pub debt: u128, // units: SEN
  pub is_initialized: bool,
}

//
// Implement Sealed trait
//
impl Sealed for Debt {}

//
// Implement IsInitialized trait
//
impl IsInitialized for Debt {
  fn is_initialized(&self) -> bool {
    self.is_initialized
  }
}

//
// Implement Pack trait
//
impl Pack for Debt {
  // Fixed length
  const LEN: usize = 113;
  // Unpack data from [u8] to the data struct
  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    msg!("Read debt data");
    let src = array_ref![src, 0, 113];
    let (stake_pool, owner, account, debt, is_initialized) = array_refs![src, 32, 32, 32, 16, 1];
    Ok(Debt {
      stake_pool: Pubkey::new_from_array(*stake_pool),
      owner: Pubkey::new_from_array(*owner),
      account: Pubkey::new_from_array(*account),
      debt: u128::from_le_bytes(*debt),
      is_initialized: match is_initialized {
        [0] => false,
        [1] => true,
        _ => return Err(ProgramError::InvalidAccountData),
      },
    })
  }
  // Pack data from the data struct to [u8]
  fn pack_into_slice(&self, dst: &mut [u8]) {
    msg!("Write debt data");
    let dst = array_mut_ref![dst, 0, 113];
    let (dst_stake_pool, dst_owner, dst_account, dst_debt, dst_is_initialized) =
      mut_array_refs![dst, 32, 32, 32, 16, 1];
    let &Debt {
      ref stake_pool,
      ref owner,
      ref account,
      debt,
      is_initialized,
    } = self;
    dst_stake_pool.copy_from_slice(stake_pool.as_ref());
    dst_owner.copy_from_slice(owner.as_ref());
    dst_account.copy_from_slice(account.as_ref());
    *dst_debt = debt.to_le_bytes();
    *dst_is_initialized = [is_initialized as u8];
  }
}
