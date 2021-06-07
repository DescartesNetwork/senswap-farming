use num_bigint::BigInt;
use num_traits::ToPrimitive;
use solana_program::msg;

const PRECISION: u64 = 1000000000000000000; // 10^18

///
/// Farming Patterns
/// Every actions can be generalize by the following pattern of flow
/// Havest -> Unstake -> Stake
///
pub struct Pattern {}

impl Pattern {
  pub fn fractionalize_reward(reward: u64, total_shares: u64) -> Option<(BigInt, BigInt)> {
    let precision = BigInt::from(PRECISION);
    if total_shares == 0 {
      return Some((BigInt::from(0u64), precision));
    }
    let reward = BigInt::from(reward);
    let total_shares = BigInt::from(total_shares);
    let fractional_reward = precision.clone() * reward.clone() / total_shares.clone();
    Some((fractional_reward, precision))
  }

  ///
  /// Havest all
  ///
  pub fn fully_havest(
    shares: u64,
    debt: u128,
    compensation: i128,
    delay: u64,
    reward: u64,
    current_total_shares: u64,
    next_total_shares: u64,
  ) -> Option<(u64, u128, i128)> {
    if current_total_shares != next_total_shares {
      return None;
    }
    // Convert to big integer
    let shares = BigInt::from(shares);
    let compensation = BigInt::from(compensation);
    let delay = BigInt::from(delay);
    // Compute current & next fraction = reward / total shares
    let (current_fraction, precision) = Self::fractionalize_reward(reward, current_total_shares)?;
    // Compute next states
    let new_debt = ((current_fraction.clone() * delay.clone() + compensation.clone())
      * shares.clone()
      / precision.clone())
    .to_u128()?;
    if debt > new_debt {
      return None;
    }
    Some((shares.to_u64()?, new_debt, compensation.to_i128()?))
  }

  ///
  /// The unstake_pattern is only called when fully havested
  ///
  pub fn fully_unstake(
    shares: u64,
    debt: u128,
    compensation: i128,
    delay: u64,
    reward: u64,
    current_total_shares: u64,
    next_total_shares: u64,
  ) -> Option<(u64, u128, i128)> {
    if next_total_shares > current_total_shares {
      return None;
    }
    // Convert to big integer
    let shares = BigInt::from(shares);
    let compensation = BigInt::from(compensation);
    let delay = BigInt::from(delay);
    // Compute current & next fraction = reward / total shares
    let (current_fraction, precision) = Self::fractionalize_reward(reward, current_total_shares)?;
    let (next_fraction, _) = Self::fractionalize_reward(reward, next_total_shares)?;
    // Whether havested
    let expected_debt = ((current_fraction.clone() * delay.clone() + compensation.clone())
      * shares.clone()
      / precision.clone())
    .to_u128()?;
    if debt != expected_debt {
      return None;
    }
    // Compute next states
    let new_compensation = if next_fraction == BigInt::from(0u64) {
      BigInt::from(0u64)
    } else {
      msg!(
        "unstake {:?} {:?} {:?}",
        compensation.to_i128()?,
        current_fraction.to_u128()?,
        next_fraction.to_u128()?
      );
      compensation.clone() - (next_fraction.clone() - current_fraction.clone()) * delay.clone()
    };
    Some((0, 0, new_compensation.to_i128()?))
  }

  ///
  /// The stake_pattern is only called when fully unstaked
  ///
  pub fn fully_stake(
    shares: u64,
    debt: u128,
    compensation: i128,
    delay: u64,
    reward: u64,
    current_total_shares: u64,
    next_total_shares: u64,
  ) -> Option<(u64, u128, i128)> {
    if current_total_shares > next_total_shares || debt != 0 {
      return None;
    }
    // Convert to big integer
    let compensation = BigInt::from(compensation);
    let delay = BigInt::from(delay);
    // Compute current & next fraction = reward / total shares
    let (current_fraction, precision) = Self::fractionalize_reward(reward, current_total_shares)?;
    let (next_fraction, _) = Self::fractionalize_reward(reward, next_total_shares)?;
    // Compute next states
    let new_compensation = if current_fraction == BigInt::from(0u64) {
      BigInt::from(0u64)
    } else {
      msg!(
        "stake {:?} {:?} {:?}",
        compensation.to_i128()?,
        current_fraction.to_u128()?,
        next_fraction.to_u128()?
      );
      compensation.clone() + (current_fraction.clone() - next_fraction.clone()) * delay.clone()
    };
    let new_debt = (next_fraction.clone() * delay.clone() + new_compensation.clone())
      * shares.clone()
      / precision.clone();
    Some((shares, new_debt.to_u128()?, new_compensation.to_i128()?))
  }
}
