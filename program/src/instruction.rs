use crate::error::AppError;
use solana_program::program_error::ProgramError;
use std::convert::TryInto;

#[derive(Clone, Debug, PartialEq)]
pub enum AppInstruction {
  InitializeStakePool { reward: u64, period: u64 },
  InitializeAccounts,
  Stake { amount: u64 },
  Unstake { amount: u64 },
  Harvest,
  FreezeStakePool,
  ThawStakePool,
  Seed { amount: u64 },
  Unseed { amount: u64 },
  TransferStakePoolOwnership,
  CloseDebt,
  CloseStakePool,
}
impl AppInstruction {
  pub fn unpack(instruction: &[u8]) -> Result<Self, ProgramError> {
    let (&tag, rest) = instruction
      .split_first()
      .ok_or(AppError::InvalidInstruction)?;
    Ok(match tag {
      0 => {
        let reward = rest
          .get(..8)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        let period = rest
          .get(8..16)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::InitializeStakePool { reward, period }
      }
      1 => Self::InitializeAccounts,
      2 => {
        let amount = rest
          .get(..8)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::Stake { amount }
      }
      3 => {
        let amount = rest
          .get(..8)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::Unstake { amount }
      }
      4 => Self::Harvest,
      5 => Self::FreezeStakePool,
      6 => Self::ThawStakePool,
      7 => {
        let amount = rest
          .get(..8)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::Seed { amount }
      }
      8 => {
        let amount = rest
          .get(..8)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::Unseed { amount }
      }
      9 => Self::TransferStakePoolOwnership,
      10 => Self::CloseDebt,
      11 => Self::CloseStakePool,
      _ => return Err(AppError::InvalidInstruction.into()),
    })
  }
}
