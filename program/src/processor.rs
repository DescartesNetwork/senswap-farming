use crate::error::AppError;
use crate::helper::{pattern::Pattern, pubutil::Boolean};
use crate::instruction::AppInstruction;
use crate::interfaces::{xsplata::XSPLATA, xsplt::XSPLT};
use crate::schema::{
  account::Account,
  debt::Debt,
  mint::Mint,
  stake_pool::{StakePool, StakePoolState},
};
use solana_program::{
  account_info::{next_account_info, AccountInfo},
  clock::Clock,
  entrypoint::ProgramResult,
  msg,
  program::{invoke, invoke_signed},
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack},
  pubkey::{Pubkey, PubkeyError},
  rent::Rent,
  system_instruction,
  sysvar::Sysvar,
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
      AppInstruction::InitializeStakePool { reward, period } => {
        msg!("Calling InitializeStakePool function");
        Self::initialize_stake_pool(reward, period, program_id, accounts)
      }

      AppInstruction::InitializeAccount {} => {
        msg!("Calling InitializeAccount function");
        Self::initialize_account(program_id, accounts)
      }

      AppInstruction::Stake { amount } => {
        msg!("Calling Stake function");
        Self::stake(amount, program_id, accounts)
      }

      AppInstruction::Unstake { amount } => {
        msg!("Calling Unstake function");
        Self::unstake(amount, program_id, accounts)
      }

      AppInstruction::Havest {} => {
        msg!("Calling Havest function");
        Self::havest(program_id, accounts)
      }

      AppInstruction::FreezeStakePool {} => {
        msg!("Calling FreezeStakePool function");
        Self::freeze_stake_pool(program_id, accounts)
      }

      AppInstruction::ThawStakePool {} => {
        msg!("Calling ThawStakePool function");
        Self::thaw_stake_pool(program_id, accounts)
      }

      AppInstruction::Seed { amount } => {
        msg!("Calling Seed function");
        Self::seed(amount, program_id, accounts)
      }

      AppInstruction::Unseed { amount } => {
        msg!("Calling Unseed function");
        Self::unseed(amount, program_id, accounts)
      }

      AppInstruction::TransferStakePoolOwnership {} => {
        msg!("Calling TransferStakePoolOwnership function");
        Self::transfer_stake_pool_ownership(program_id, accounts)
      }
    }
  }

  pub fn initialize_stake_pool(
    reward: u64,
    period: u64,
    program_id: &Pubkey,
    accounts: &[AccountInfo],
  ) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let owner = next_account_info(accounts_iter)?;
    let stake_pool_acc = next_account_info(accounts_iter)?;
    let mint_share_acc = next_account_info(accounts_iter)?;
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
      return Err(AppError::UnmatchedPool.into());
    }
    if reward == 0 {
      return Err(AppError::ZeroValue.into());
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

    // Initialize treasury sen
    XSPLATA::initialize_account(
      payer,
      treasury_sen_acc,
      treasurer,
      mint_sen_acc,
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

    // Update stake pool data
    stake_pool_data.owner = *owner.key;
    stake_pool_data.state = StakePoolState::Initialized;
    stake_pool_data.genesis_timestamp = Self::current_timestamp()?;
    stake_pool_data.total_shares = 0;
    stake_pool_data.mint_share = *mint_share_acc.key;
    stake_pool_data.mint_token = *mint_token_acc.key;
    stake_pool_data.treasury_token = *treasury_token_acc.key;
    stake_pool_data.reward = reward;
    stake_pool_data.period = period;
    stake_pool_data.compensation = 0;
    stake_pool_data.treasury_sen = *treasury_sen_acc.key;
    StakePool::pack(stake_pool_data, &mut stake_pool_acc.data.borrow_mut())?;

    Ok(())
  }

  pub fn initialize_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let owner = next_account_info(accounts_iter)?;
    let stake_pool_acc = next_account_info(accounts_iter)?;
    let mint_share_acc = next_account_info(accounts_iter)?;

    let share_acc = next_account_info(accounts_iter)?;
    let debt_acc = next_account_info(accounts_iter)?;

    let system_program = next_account_info(accounts_iter)?;
    let splt_program = next_account_info(accounts_iter)?;
    let sysvar_rent_acc = next_account_info(accounts_iter)?;
    let splata_program = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[stake_pool_acc])?;
    Self::is_signer(&[payer])?;

    StakePool::unpack(&stake_pool_acc.data.borrow())?;

    // Initilized share account
    XSPLATA::initialize_account(
      payer,
      share_acc,
      owner,
      mint_share_acc,
      system_program,
      splt_program,
      sysvar_rent_acc,
      splata_program,
      &[],
    )?;

    // Validate debt account address
    let (key, bump_seed) = Pubkey::find_program_address(
      &[
        &owner.key.to_bytes(),
        &stake_pool_acc.key.to_bytes(),
        &program_id.to_bytes(),
      ],
      program_id,
    );
    let seed: &[&[u8]] = &[
      &owner.key.to_bytes(),
      &stake_pool_acc.key.to_bytes(),
      &program_id.to_bytes(),
      &[bump_seed],
    ];
    if key != *debt_acc.key {
      return Err(AppError::InvalidOwner.into());
    }
    // Rent space
    let rent = &Rent::from_account_info(sysvar_rent_acc)?;
    let required_lamports = rent
      .minimum_balance(Debt::LEN)
      .max(1)
      .saturating_sub(debt_acc.lamports());
    if required_lamports > 0 {
      invoke(
        &system_instruction::transfer(&payer.key, debt_acc.key, required_lamports),
        &[payer.clone(), debt_acc.clone(), system_program.clone()],
      )?;
    }
    // Allocate space
    invoke_signed(
      &system_instruction::allocate(debt_acc.key, Debt::LEN as u64),
      &[debt_acc.clone(), system_program.clone()],
      &[&seed],
    )?;
    // Assign owner to farming program
    invoke_signed(
      &system_instruction::assign(debt_acc.key, &program_id),
      &[debt_acc.clone(), system_program.clone()],
      &[&seed],
    )?;

    // Assign data
    let mut debt_data = Debt::unpack_unchecked(&debt_acc.data.borrow())?;
    if debt_data.is_initialized() {
      return Err(AppError::ConstructorOnce.into());
    }
    debt_data.stake_pool = *stake_pool_acc.key;
    debt_data.owner = *owner.key;
    debt_data.account = *share_acc.key;
    debt_data.debt = 0;
    debt_data.is_initialized = true;
    Debt::pack(debt_data, &mut debt_acc.data.borrow_mut())?;

    Ok(())
  }

  pub fn stake(amount: u64, program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let stake_pool_acc = next_account_info(accounts_iter)?;
    let mint_share_acc = next_account_info(accounts_iter)?;

    let src_acc = next_account_info(accounts_iter)?;
    let treasury_token_acc = next_account_info(accounts_iter)?;

    let share_acc = next_account_info(accounts_iter)?;
    let debt_acc = next_account_info(accounts_iter)?;

    let dst_sen_acc = next_account_info(accounts_iter)?;
    let treasury_sen_acc = next_account_info(accounts_iter)?;

    let treasurer = next_account_info(accounts_iter)?;
    let splt_program = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[stake_pool_acc, debt_acc])?;
    Self::is_signer(&[owner])?;
    Self::is_debt_owner(owner, debt_acc, stake_pool_acc, share_acc)?;

    let mut stake_pool_data = StakePool::unpack(&stake_pool_acc.data.borrow())?;
    let share_data = Account::unpack(&share_acc.data.borrow())?;
    let mut debt_data = Debt::unpack(&debt_acc.data.borrow())?;
    let seed: &[&[&[u8]]] = &[&[&Self::safe_seed(stake_pool_acc, treasurer, program_id)?[..]]];
    if stake_pool_data.mint_share != *mint_share_acc.key
      || stake_pool_data.treasury_token != *treasury_token_acc.key
    {
      return Err(AppError::UnmatchedPool.into());
    }
    if stake_pool_data.is_frozen() {
      return Err(AppError::FrozenPool.into());
    }
    if amount == 0 {
      return Err(AppError::ZeroValue.into());
    }

    // Stake token
    XSPLT::transfer(
      amount,
      src_acc,
      treasury_token_acc,
      owner,
      splt_program,
      &[],
    )?;

    // Get the basics
    let shares = share_data.amount;
    let debt = debt_data.debt;
    let compensation = stake_pool_data.compensation;
    let delay = Self::estimate_delay(stake_pool_data)?;
    let reward = stake_pool_data.reward;
    msg!("Debug: delay={:?} reward={:?}", delay, reward);
    let current_total_shares = stake_pool_data.total_shares;
    msg!(
      "Debug: (starting) state = ({:?}, {:?}, {:?})",
      shares,
      debt,
      compensation
    );
    // Fully havest
    let next_total_shares = current_total_shares; // Havest doesn't change the total shares
    msg!(
      "Debug: total shares = ({:?}, {:?})",
      current_total_shares,
      next_total_shares
    );
    let (shares, debt, compensation) = Pattern::fully_havest(
      shares,
      debt,
      compensation,
      delay,
      reward,
      current_total_shares,
      next_total_shares,
    )
    .ok_or(AppError::Overflow)?;
    msg!(
      "Debug: (after fully havest) state = ({:?}, {:?}, {:?})",
      shares,
      debt,
      compensation
    );
    let yeild = debt.checked_sub(debt_data.debt).ok_or(AppError::Overflow)? as u64;
    msg!("Debug: yeild = {:?}", yeild);
    // Fully unstake
    let next_total_shares = current_total_shares
      .checked_sub(shares)
      .ok_or(AppError::Overflow)?;
    msg!(
      "Debug: total shares = ({:?}, {:?})",
      current_total_shares,
      next_total_shares
    );
    let (shares, debt, compensation) = Pattern::fully_unstake(
      shares,
      debt,
      compensation,
      delay,
      reward,
      current_total_shares,
      next_total_shares,
    )
    .ok_or(AppError::Overflow)?;
    msg!(
      "Debug: (after fully unstake) state = ({:?}, {:?}, {:?})",
      shares,
      debt,
      compensation
    );
    // Fully stake
    let shares = share_data
      .amount
      .checked_add(amount)
      .ok_or(AppError::Overflow)?;
    let current_total_shares = next_total_shares;
    let next_total_shares = current_total_shares
      .checked_add(shares)
      .ok_or(AppError::Overflow)?;
    msg!(
      "Debug: total shares = ({:?}, {:?})",
      current_total_shares,
      next_total_shares
    );
    let (shares, debt, compensation) = Pattern::fully_stake(
      shares,
      debt,
      compensation,
      delay,
      reward,
      current_total_shares,
      next_total_shares,
    )
    .ok_or(AppError::Overflow)?;
    msg!(
      "Debug: (after fully stake) state = ({:?}, {:?}, {:?})",
      shares,
      debt,
      compensation
    );

    // Havest
    XSPLT::transfer(
      yeild,
      treasury_sen_acc,
      dst_sen_acc,
      treasurer,
      splt_program,
      seed,
    )?;
    // Mint share
    XSPLT::mint_to(
      amount,
      mint_share_acc,
      share_acc,
      treasurer,
      splt_program,
      seed,
    )?;

    // Debt account
    debt_data.debt = debt;
    Debt::pack(debt_data, &mut debt_acc.data.borrow_mut())?;
    // Stake pool account
    stake_pool_data.total_shares = next_total_shares;
    stake_pool_data.compensation = compensation;
    StakePool::pack(stake_pool_data, &mut stake_pool_acc.data.borrow_mut())?;

    Ok(())
  }

  pub fn unstake(amount: u64, program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let stake_pool_acc = next_account_info(accounts_iter)?;
    let mint_share_acc = next_account_info(accounts_iter)?;

    let dst_acc = next_account_info(accounts_iter)?;
    let treasury_token_acc = next_account_info(accounts_iter)?;

    let share_acc = next_account_info(accounts_iter)?;
    let debt_acc = next_account_info(accounts_iter)?;

    let dst_sen_acc = next_account_info(accounts_iter)?;
    let treasury_sen_acc = next_account_info(accounts_iter)?;

    let treasurer = next_account_info(accounts_iter)?;
    let splt_program = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[stake_pool_acc, debt_acc])?;
    Self::is_signer(&[owner])?;
    Self::is_debt_owner(owner, debt_acc, stake_pool_acc, share_acc)?;

    let mut stake_pool_data = StakePool::unpack(&stake_pool_acc.data.borrow())?;
    let share_data = Account::unpack(&share_acc.data.borrow())?;
    let mut debt_data = Debt::unpack(&debt_acc.data.borrow())?;
    let seed: &[&[&[u8]]] = &[&[&Self::safe_seed(stake_pool_acc, treasurer, program_id)?[..]]];
    if stake_pool_data.mint_share != *mint_share_acc.key
      || stake_pool_data.treasury_token != *treasury_token_acc.key
      || stake_pool_data.treasury_sen != *treasury_sen_acc.key
    {
      return Err(AppError::UnmatchedPool.into());
    }
    if stake_pool_data.is_frozen() {
      return Err(AppError::FrozenPool.into());
    }
    if amount == 0 {
      return Err(AppError::ZeroValue.into());
    }

    // Get the basics
    let shares = share_data.amount;
    let debt = debt_data.debt;
    let compensation = stake_pool_data.compensation;
    let delay = Self::estimate_delay(stake_pool_data)?;
    let reward = stake_pool_data.reward;
    msg!("Debug: delay={:?} reward={:?}", delay, reward);
    let current_total_shares = stake_pool_data.total_shares;
    msg!(
      "Debug: (starting) state = ({:?}, {:?}, {:?})",
      shares,
      debt,
      compensation
    );
    // Fully havest
    let next_total_shares = current_total_shares; // Havest all before unstaking
    msg!(
      "Debug: total shares = ({:?}, {:?})",
      current_total_shares,
      next_total_shares
    );
    let (shares, debt, compensation) = Pattern::fully_havest(
      shares,
      debt,
      compensation,
      delay,
      reward,
      current_total_shares,
      next_total_shares,
    )
    .ok_or(AppError::Overflow)?;
    msg!(
      "Debug: (after fully havest) state = ({:?}, {:?}, {:?})",
      shares,
      debt,
      compensation
    );
    let yeild = debt.checked_sub(debt_data.debt).ok_or(AppError::Overflow)? as u64;
    msg!("Debug: yeild = {:?}", yeild);
    // Fully unstake
    let next_total_shares = current_total_shares
      .checked_sub(shares)
      .ok_or(AppError::Overflow)?;
    msg!(
      "Debug: total shares = ({:?}, {:?})",
      current_total_shares,
      next_total_shares
    );
    let (shares, debt, compensation) = Pattern::fully_unstake(
      shares,
      debt,
      compensation,
      delay,
      reward,
      current_total_shares,
      next_total_shares,
    )
    .ok_or(AppError::Overflow)?;
    msg!(
      "Debug: (after fully unstake) state = ({:?}, {:?}, {:?})",
      shares,
      debt,
      compensation
    );
    // Fully stake
    let shares = share_data
      .amount
      .checked_sub(amount)
      .ok_or(AppError::Overflow)?;
    let current_total_shares = next_total_shares;
    let next_total_shares = current_total_shares
      .checked_add(shares)
      .ok_or(AppError::Overflow)?;
    msg!(
      "Debug: total shares = ({:?}, {:?})",
      current_total_shares,
      next_total_shares
    );
    let (shares, debt, compensation) = Pattern::fully_stake(
      shares,
      debt,
      compensation,
      delay,
      reward,
      current_total_shares,
      next_total_shares,
    )
    .ok_or(AppError::Overflow)?;
    msg!(
      "Debug: (after fully stake) state = ({:?}, {:?}, {:?})",
      shares,
      debt,
      compensation
    );

    // Havest
    XSPLT::transfer(
      yeild,
      treasury_sen_acc,
      dst_sen_acc,
      treasurer,
      splt_program,
      seed,
    )?;
    // Unstake token
    XSPLT::burn(
      amount,
      share_acc,
      mint_share_acc,
      owner,
      splt_program,
      &[],
    )?;
    XSPLT::transfer(
      amount,
      treasury_token_acc,
      dst_acc,
      treasurer,
      splt_program,
      seed,
    )?;

    // Debt account
    debt_data.debt = debt;
    Debt::pack(debt_data, &mut debt_acc.data.borrow_mut())?;
    // Stake pool account
    stake_pool_data.total_shares = next_total_shares;
    stake_pool_data.compensation = compensation;
    StakePool::pack(stake_pool_data, &mut stake_pool_acc.data.borrow_mut())?;

    Ok(())
  }

  pub fn havest(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let stake_pool_acc = next_account_info(accounts_iter)?;
    let mint_share_acc = next_account_info(accounts_iter)?;

    let share_acc = next_account_info(accounts_iter)?;
    let debt_acc = next_account_info(accounts_iter)?;

    let dst_sen_acc = next_account_info(accounts_iter)?;
    let treasury_sen_acc = next_account_info(accounts_iter)?;

    let treasurer = next_account_info(accounts_iter)?;
    let splt_program = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[stake_pool_acc, debt_acc])?;
    Self::is_signer(&[owner])?;
    Self::is_debt_owner(owner, debt_acc, stake_pool_acc, share_acc)?;

    let mut stake_pool_data = StakePool::unpack(&stake_pool_acc.data.borrow())?;
    let share_data = Account::unpack(&share_acc.data.borrow())?;
    let mut debt_data = Debt::unpack(&debt_acc.data.borrow())?;
    let seed: &[&[&[u8]]] = &[&[&Self::safe_seed(stake_pool_acc, treasurer, program_id)?[..]]];
    if stake_pool_data.mint_share != *mint_share_acc.key
      || stake_pool_data.treasury_sen != *treasury_sen_acc.key
    {
      return Err(AppError::UnmatchedPool.into());
    }
    if stake_pool_data.is_frozen() {
      return Err(AppError::FrozenPool.into());
    }

    // Get the basics
    let shares = share_data.amount;
    let debt = debt_data.debt;
    let compensation = stake_pool_data.compensation;
    let delay = Self::estimate_delay(stake_pool_data)?;
    let reward = stake_pool_data.reward;
    msg!("Debug: delay={:?} reward={:?}", delay, reward);
    let current_total_shares = stake_pool_data.total_shares;
    msg!(
      "Debug: (starting) state = ({:?}, {:?}, {:?})",
      shares,
      debt,
      compensation
    );
    // Fully havest
    let next_total_shares = current_total_shares; // Havest doesn't change the total shares
    msg!(
      "Debug: total shares = ({:?}, {:?})",
      current_total_shares,
      next_total_shares
    );
    let (_shares, debt, compensation) = Pattern::fully_havest(
      shares,
      debt,
      compensation,
      delay,
      reward,
      current_total_shares,
      next_total_shares,
    )
    .ok_or(AppError::Overflow)?;
    msg!(
      "Debug: (after fully havest) state = ({:?}, {:?}, {:?})",
      shares,
      debt,
      compensation
    );
    let yeild = debt.checked_sub(debt_data.debt).ok_or(AppError::Overflow)? as u64;
    msg!("Debug: yeild = {:?}", yeild);

    // Havest
    XSPLT::transfer(
      yeild,
      treasury_sen_acc,
      dst_sen_acc,
      treasurer,
      splt_program,
      seed,
    )?;

    // Debt account
    debt_data.debt = debt;
    Debt::pack(debt_data, &mut debt_acc.data.borrow_mut())?;
    // Stake pool account
    stake_pool_data.total_shares = next_total_shares;
    stake_pool_data.compensation = compensation;
    StakePool::pack(stake_pool_data, &mut stake_pool_acc.data.borrow_mut())?;

    Ok(())
  }

  pub fn freeze_stake_pool(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
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

  pub fn thaw_stake_pool(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
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
      treasurer,
      splt_program,
      seed,
    )?;

    Ok(())
  }

  pub fn transfer_stake_pool_ownership(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
  ) -> ProgramResult {
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

  pub fn is_debt_owner(
    owner: &AccountInfo,
    debt_acc: &AccountInfo,
    stake_pool_acc: &AccountInfo,
    share_acc: &AccountInfo,
  ) -> ProgramResult {
    let debt_data = Debt::unpack(&debt_acc.data.borrow())?;
    if debt_data.stake_pool != *stake_pool_acc.key
      || debt_data.owner != *owner.key
      || debt_data.account != *share_acc.key
    {
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

  pub fn current_timestamp() -> Result<i64, ProgramError> {
    let clock = Clock::get()?;
    Ok(clock.unix_timestamp)
  }

  pub fn estimate_delay(stake_pool_data: StakePool) -> Result<u64, ProgramError> {
    let current_timestamp = Self::current_timestamp()?;
    let delay =
      (current_timestamp - stake_pool_data.genesis_timestamp) as u64 / stake_pool_data.period;
    Ok(delay)
  }
}
