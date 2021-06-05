use num_bigint::BigUint;
use num_traits::ToPrimitive;

const PRECISION: u64 = 1000000000000000000; // 10^18
const FEE: u64 = 2500000; // 0.25%
const EARNING: u64 = 500000; // 0.05%
const DECIMALS: u64 = 1000000000; // 10^9

///
/// Farming Patterns
/// Every actions can be generalize by the following pattern of flow
/// Havest -> Unstake -> Stake
///
pub struct Pattern {}

impl Pattern {
  pub fn fractional_reward(reward: u64, total_shares: u64) -> Option<(BigUint, BigUint)> {
    let precision = BigUint::from(PRECISION);
    let reward = BigUint::from(reward);
    let total_shares = BigUint::from(total_shares);
    if total_shares == BigUint::from(0u64) {
      return None;
    }
    let fractional_reward = precision.clone() * reward / total_shares;
    Some((fractional_reward, precision))
  }

  ///
  /// Havest all
  ///
  pub fn fully_havest(
    shares: u64,
    debt: u64,
    compensation: u64,
    delay: u64,
    reward: u64,
    current_total_shares: u64,
    next_total_shares: u64,
  ) -> Option<(u64, u64, u64)> {
    if current_total_shares != next_total_shares {
      return None;
    }
    // Convert to big integer
    let shares = BigUint::from(shares);
    let compensation = BigUint::from(compensation);
    let delay = BigUint::from(delay);
    // Compute current & next fraction = reward / total shares
    let (current_fraction, precision) = Self::fractional_reward(reward, current_total_shares)?;
    // Compute next states
    let new_shares = shares.to_u64()?;
    let new_debt = ((current_fraction.clone() * delay.clone() + compensation.clone())
      * shares.clone()
      / precision.clone())
    .to_u64()?;
    if debt > new_debt {
      return None;
    }
    let new_compensation = compensation.to_u64()?;
    Some((new_shares, new_debt, new_compensation))
  }

  ///
  /// The unstake_pattern is only called when fully havested
  ///
  pub fn fully_unstake(
    shares: u64,
    debt: u64,
    compensation: u64,
    delay: u64,
    reward: u64,
    current_total_shares: u64,
    next_total_shares: u64,
  ) -> Option<(u64, u64, u64)> {
    if next_total_shares > current_total_shares {
      return None;
    }
    // Convert to big integer
    let shares = BigUint::from(shares);
    let compensation = BigUint::from(compensation);
    let delay = BigUint::from(delay);
    // Compute current & next fraction = reward / total shares
    let (current_fraction, precision) = Self::fractional_reward(reward, current_total_shares)?;
    let (next_fraction, _) = Self::fractional_reward(reward, next_total_shares)?;
    // Whether havested
    let expected_debt = ((current_fraction.clone() * delay.clone() + compensation.clone())
      * shares.clone()
      / precision.clone())
    .to_u64()?;
    if debt != expected_debt {
      return None;
    }
    // Compute next states
    let new_compensation = (compensation.clone()
      - (next_fraction.clone() - current_fraction.clone()) * delay.clone() / precision.clone())
    .to_u64()?;
    Some((0, 0, new_compensation))
  }

  ///
  /// The stake_pattern is only called when fully unstaked
  ///
  pub fn fully_stake(
    shares: u64,
    debt: u64,
    compensation: u64,
    delay: u64,
    reward: u64,
    current_total_shares: u64,
    next_total_shares: u64,
  ) -> Option<(u64, u64, u64)> {
    if current_total_shares > next_total_shares || debt != 0 {
      return None;
    }
    // Convert to big integer
    let compensation = BigUint::from(compensation);
    let delay = BigUint::from(delay);
    // Compute current & next fraction = reward / total shares
    let (current_fraction, precision) = Self::fractional_reward(reward, current_total_shares)?;
    let (next_fraction, _) = Self::fractional_reward(reward, next_total_shares)?;
    // Compute next states
    let new_compensation =
      compensation.clone() + (current_fraction.clone() - next_fraction.clone()) * delay.clone();
    let new_debt = ((next_fraction.clone() * delay.clone() + new_compensation.clone())
      * shares.clone()
      / precision.clone())
    .to_u64()?;
    let new_compensation = (new_compensation.clone() / precision.clone()).to_u64()?;
    Some((shares, new_debt, new_compensation))
  }
}
