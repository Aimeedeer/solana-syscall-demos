use borsh::de::BorshDeserialize;
use common::{CustomInstruction, PrintSysvarsInstruction};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

mod secp256k1;

entrypoint!(process_instruction);

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = {
        let mut instruction_data = instruction_data;
        CustomInstruction::deserialize(&mut instruction_data)?
    };

    match instruction {
        CustomInstruction::PrintSysvars(instr) => {
            print_sysvars(instr, accounts, instruction_data)?;
        }
        CustomInstruction::DemoSecp256k1Basic(instr) => {
            secp256k1::demo_secp256k1_basic(instr, accounts)?;
        }
        CustomInstruction::DemoSecp256k1Recover(instr) => {
            secp256k1::demo_secp256k1_recover(instr, accounts)?;
        }
    }

    Ok(())
}

fn print_sysvars(
    _instruction: PrintSysvarsInstruction,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("sysvar program printing");

    let account_info_iter = &mut accounts.iter();

    let _system_program_account = next_account_info(account_info_iter)?;

    let clock_account = next_account_info(account_info_iter)?;
    let epoch_schedule_account = next_account_info(account_info_iter)?;

    let instructions_account = next_account_info(account_info_iter)?;
    let rent_account = next_account_info(account_info_iter)?;

    let slot_hashes_account = next_account_info(account_info_iter)?;
    let slot_history_account = next_account_info(account_info_iter)?;

    let stake_history_account = next_account_info(account_info_iter)?;

    // `Program consumed 200000 of 200000 compute units`
    // comment some blocks of code to run it correctly

    {
        use solana_program::sysvar::clock::{self, Clock};

        assert!(clock::check_id(clock_account.key));
        assert_eq!(*clock_account.key, clock::ID);

        let clock_from_account = Clock::from_account_info(clock_account)?;
        let clock_from_sysvar = Clock::get()?;

        assert_eq!(clock_from_account, clock_from_sysvar);

        msg!("clock: {:#?}", clock_from_account);
    }

    {
        use solana_program::sysvar::epoch_schedule::{self, EpochSchedule};

        assert!(epoch_schedule::check_id(epoch_schedule_account.key));
        assert_eq!(*epoch_schedule_account.key, epoch_schedule::ID);

        let epoch_schedule_from_account = EpochSchedule::from_account_info(epoch_schedule_account)?;
        let epoch_schedule_from_sysvar = EpochSchedule::get()?;

        assert_eq!(epoch_schedule_from_account, epoch_schedule_from_sysvar);

        msg!("epoch_schedule: {:#?}", epoch_schedule_from_sysvar);
    }

    {
        use solana_program::sysvar::instructions;

        assert!(instructions::check_id(instructions_account.key));
        assert_eq!(*instructions_account.key, instructions::ID);

        let current_index = instructions::load_current_index_checked(instructions_account)?;
        let instructions_from_account =
            instructions::load_instruction_at_checked(current_index.into(), instructions_account)?;

        assert_eq!(instructions_from_account.data, instruction_data);

        let mut instruction_data = instruction_data;
        let deserialized_instruction_data = CustomInstruction::deserialize(&mut instruction_data)?;

        msg!(
            "deserialized_instruction_data: {:#?}",
            deserialized_instruction_data
        );
    }

    {
        use solana_program::sysvar::rent::{self, Rent};

        assert!(rent::check_id(rent_account.key));
        assert_eq!(*rent_account.key, rent::ID);

        let rent_from_account = Rent::from_account_info(rent_account)?;
        let rent_from_sysvar = Rent::get()?;

        assert_eq!(rent_from_account, rent_from_sysvar);

        msg!("rent: {:#?}", rent_from_account);
    }

    {
        use solana_program::sysvar::slot_hashes;

        assert!(slot_hashes::check_id(slot_hashes_account.key));
        assert_eq!(*slot_hashes_account.key, slot_hashes::ID);

        msg!("slot_hashes: {:#?}", slot_hashes_account);
    }

    {
        use solana_program::sysvar::slot_history;

        assert!(slot_history::check_id(slot_history_account.key));
        assert_eq!(*slot_history_account.key, slot_history::ID);

        msg!("slot_history: {:#?}", slot_history_account);
    }

    {
        use solana_program::sysvar::stake_history;

        assert!(stake_history::check_id(stake_history_account.key));
        assert_eq!(*stake_history_account.key, stake_history::ID);

        msg!("stake_history: {:#?}", stake_history_account);
    }

    Ok(())
}
