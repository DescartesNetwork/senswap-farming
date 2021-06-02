use crate::error::AppError;
use crate::helper::pubutil::Boolean;
use crate::instruction::AppInstruction;
use crate::interfaces::{xsplata::XSPLATA, xsplt::XSPLT};
use crate::schema::{
  mint::Mint,
  stake_pool::{StakePool, StakePoolState},
};
use solana_program::{
  account_info::{next_account_info, AccountInfo},
  entrypoint::ProgramResult,
  msg,
  program_pack::{IsInitialized, Pack},
  pubkey::{Pubkey, PubkeyError},
};

pub struct Processor {}

impl Processor {
  pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
  ) -> ProgramResult {
    let instruction = AppInstruction::unpack(instruction_data)?;
    match instruction {
      AppInstruction::InitializeStakePool { reward } => {
        msg!("Calling InitializeStakePool function");
        Self::initialize_stake_pool(reward, program_id, accounts)
      }

      AppInstruction::Stake { amount } => {
        msg!("Calling Stake function");
        Self::stake(amount, program_id, accounts)
      }

      AppInstruction::Unstake { amount } => {
        msg!("Calling Unstake function");
        Self::unstake(amount, program_id, accounts)
      }

      AppInstruction::Havest { amount } => {
        msg!("Calling Havest function");
        Self::havest(amount, program_id, accounts)
      }

      AppInstruction::FreezePool {} => {
        msg!("Calling FreezePool function");
        Self::freeze_pool(program_id, accounts)
      }

      AppInstruction::ThawPool {} => {
        msg!("Calling ThawPool function");
        Self::thaw_pool(program_id, accounts)
      }

      AppInstruction::Seed { amount } => {
        msg!("Calling Seed function");
        Self::seed(amount, program_id, accounts)
      }

      AppInstruction::Unseed { amount } => {
        msg!("Calling Unseed function");
        Self::unseed(amount, program_id, accounts)
      }

      AppInstruction::Earn { amount } => {
        msg!("Calling Earn function");
        Self::earn(amount, program_id, accounts)
      }

      AppInstruction::TransferPoolOwnership {} => {
        msg!("Calling TransferPoolOwnership function");
        Self::transfer_pool_ownership(program_id, accounts)
      }
    }
  }

  pub fn initialize_stake_pool(
    reward: u64,
    program_id: &Pubkey,
    accounts: &[AccountInfo],
  ) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let owner = next_account_info(accounts_iter)?;
    let stake_pool_acc = next_account_info(accounts_iter)?;
    let mint_share_acc = next_account_info(accounts_iter)?;
    let vault_acc = next_account_info(accounts_iter)?;
    let proof_acc = next_account_info(accounts_iter)?; // program_id xor treasurer xor stake_pool_id

    let mint_token_acc = next_account_info(accounts_iter)?;
    let treasury_token_acc = next_account_info(accounts_iter)?;

    let mint_sen_acc = next_account_info(accounts_iter)?;
    let treasury_sen_acc = next_account_info(accounts_iter)?;
    let treasurer = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let splt_program = next_account_info(accounts_iter)?;
    let sysvar_rent_acc = next_account_info(accounts_iter)?;
    let splata_program = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[stake_pool_acc])?;
    Self::is_signer(&[payer, stake_pool_acc])?;

    let mut stake_pool_data = StakePool::unpack_unchecked(&stake_pool_acc.data.borrow())?;
    let mint_share_data = Mint::unpack_unchecked(&mint_share_acc.data.borrow())?;
    let seed: &[&[&[u8]]] = &[&[&Self::safe_seed(stake_pool_acc, treasurer, program_id)?[..]]];
    if stake_pool_data.is_initialized() || mint_share_data.is_initialized() {
      return Err(AppError::ConstructorOnce.into());
    }
    if *proof_acc.key != program_id.xor(&(stake_pool_acc.key.xor(treasurer.key))) {
      return Err(AppError::InvalidMint.into());
    }

    // Initialize treasury token
    XSPLATA::initialize_account(
      payer,
      treasury_token_acc,
      treasurer,
      mint_token_acc,
      system_program,
      splt_program,
      sysvar_rent_acc,
      splata_program,
      seed,
    )?;

    // Initialize mint share
    let mint_token_data = Mint::unpack_unchecked(&mint_token_acc.data.borrow())?;
    XSPLT::initialize_mint(
      mint_token_data.decimals,
      mint_share_acc,
      treasurer,
      proof_acc,
      sysvar_rent_acc,
      splt_program,
      seed,
    )?;

    // Initialize vault
    XSPLT::initialize_account(
      vault_acc,
      mint_sen_acc,
      treasurer,
      sysvar_rent_acc,
      splt_program,
      seed,
    )?;

    // Update stake pool data
    stake_pool_data.owner = *owner.key;
    stake_pool_data.state = StakePoolState::Initialized;
    stake_pool_data.mint_share = *mint_share_acc.key;
    stake_pool_data.vault = *vault_acc.key;
    stake_pool_data.total_token_locked = 0;
    stake_pool_data.mint_token = *mint_token_acc.key;
    stake_pool_data.treasury_token = *treasury_token_acc.key;
    stake_pool_data.reward = reward;
    stake_pool_data.debt = 0;
    stake_pool_data.treasury_sen = *treasury_sen_acc.key;

    Ok(())
  }

  pub fn stake(amount: u64, program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let stake_pool_acc = next_account_info(accounts_iter)?;

    Ok(())
  }

  pub fn unstake(amount: u64, program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let stake_pool_acc = next_account_info(accounts_iter)?;

    Ok(())
  }

  pub fn havest(amount: u64, program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let stake_pool_acc = next_account_info(accounts_iter)?;

    Ok(())
  }

  pub fn freeze_pool(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let stake_pool_acc = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[stake_pool_acc])?;
    Self::is_signer(&[owner])?;
    Self::is_stake_pool_owner(owner, stake_pool_acc)?;

    let mut stake_pool_data = StakePool::unpack(&stake_pool_acc.data.borrow())?;
    stake_pool_data.state = StakePoolState::Frozen;
    StakePool::pack(stake_pool_data, &mut stake_pool_acc.data.borrow_mut())?;

    Ok(())
  }

  pub fn thaw_pool(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let stake_pool_acc = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[stake_pool_acc])?;
    Self::is_signer(&[owner])?;
    Self::is_stake_pool_owner(owner, stake_pool_acc)?;

    let mut stake_pool_data = StakePool::unpack(&stake_pool_acc.data.borrow())?;
    stake_pool_data.state = StakePoolState::Initialized;
    StakePool::pack(stake_pool_data, &mut stake_pool_acc.data.borrow_mut())?;

    Ok(())
  }

  pub fn seed(amount: u64, program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let stake_pool_acc = next_account_info(accounts_iter)?;
    let src_sen_acc = next_account_info(accounts_iter)?;
    let treasury_sen_acc = next_account_info(accounts_iter)?;
    let splt_program = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[stake_pool_acc])?;
    Self::is_signer(&[owner])?;

    let stake_pool_data = StakePool::unpack(&stake_pool_acc.data.borrow())?;
    if stake_pool_data.treasury_sen != *treasury_sen_acc.key {
      return Err(AppError::UnmatchedPool.into());
    }
    if amount == 0 {
      return Err(AppError::ZeroValue.into());
    }

    // Deposit SEN to treasury
    XSPLT::transfer(
      amount,
      src_sen_acc,
      treasury_sen_acc,
      owner,
      splt_program,
      &[],
    )?;

    Ok(())
  }

  pub fn unseed(amount: u64, program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let stake_pool_acc = next_account_info(accounts_iter)?;
    let dst_sen_acc = next_account_info(accounts_iter)?;
    let treasury_sen_acc = next_account_info(accounts_iter)?;
    let treasurer = next_account_info(accounts_iter)?;
    let splt_program = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[stake_pool_acc])?;
    Self::is_signer(&[owner])?;
    Self::is_stake_pool_owner(owner, stake_pool_acc)?;

    let stake_pool_data = StakePool::unpack(&stake_pool_acc.data.borrow())?;
    let seed: &[&[&[u8]]] = &[&[&Self::safe_seed(stake_pool_acc, treasurer, program_id)?[..]]];
    if stake_pool_data.treasury_sen != *treasury_sen_acc.key {
      return Err(AppError::UnmatchedPool.into());
    }
    if amount == 0 {
      return Err(AppError::ZeroValue.into());
    }

    // Withdraw SEN to treasury
    XSPLT::transfer(
      amount,
      treasury_sen_acc,
      dst_sen_acc,
      owner,
      splt_program,
      seed,
    )?;

    Ok(())
  }

  pub fn earn(amount: u64, program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let stake_pool_acc = next_account_info(accounts_iter)?;
    let vault_acc = next_account_info(accounts_iter)?;
    let dst_acc = next_account_info(accounts_iter)?;
    let treasurer = next_account_info(accounts_iter)?;
    let splt_program = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[stake_pool_acc])?;
    Self::is_signer(&[owner])?;
    Self::is_stake_pool_owner(owner, stake_pool_acc)?;

    let stake_pool_data = StakePool::unpack(&stake_pool_acc.data.borrow())?;
    let seed: &[&[&[u8]]] = &[&[&Self::safe_seed(stake_pool_acc, treasurer, program_id)?[..]]];
    if stake_pool_data.vault != *vault_acc.key {
      return Err(AppError::InvalidOwner.into());
    }
    if amount == 0 {
      return Err(AppError::ZeroValue.into());
    }
    // Transfer earning
    XSPLT::transfer(amount, vault_acc, dst_acc, treasurer, splt_program, seed)?;

    Ok(())
  }

  pub fn transfer_pool_ownership(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let stake_pool_acc = next_account_info(accounts_iter)?;
    let new_owner = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[stake_pool_acc])?;
    Self::is_signer(&[owner])?;
    Self::is_stake_pool_owner(owner, stake_pool_acc)?;

    // Update stake pool data
    let mut stake_pool_data = StakePool::unpack(&stake_pool_acc.data.borrow())?;
    stake_pool_data.owner = *new_owner.key;
    StakePool::pack(stake_pool_data, &mut stake_pool_acc.data.borrow_mut())?;

    Ok(())
  }

  ///
  /// Utilities
  ///

  pub fn is_program(program_id: &Pubkey, accounts: &[&AccountInfo]) -> ProgramResult {
    for acc in &mut accounts.iter() {
      if acc.owner != program_id {
        return Err(AppError::IncorrectProgramId.into());
      }
    }
    Ok(())
  }

  pub fn is_signer(accounts: &[&AccountInfo]) -> ProgramResult {
    for acc in &mut accounts.iter() {
      if !acc.is_signer {
        return Err(AppError::InvalidOwner.into());
      }
    }
    Ok(())
  }

  pub fn is_stake_pool_owner(owner: &AccountInfo, stake_pool_acc: &AccountInfo) -> ProgramResult {
    let stake_pool_data = StakePool::unpack(&stake_pool_acc.data.borrow())?;
    if stake_pool_data.owner != *owner.key {
      return Err(AppError::InvalidOwner.into());
    }
    Ok(())
  }

  pub fn safe_seed(
    seed_acc: &AccountInfo,
    expected_acc: &AccountInfo,
    program_id: &Pubkey,
  ) -> Result<[u8; 32], PubkeyError> {
    let seed: [u8; 32] = seed_acc.key.to_bytes();
    let key = Pubkey::create_program_address(&[&seed], program_id)?;
    if key != *expected_acc.key {
      return Err(PubkeyError::InvalidSeeds);
    }
    Ok(seed)
  }
}
