use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint {
    use super::process_instruction;
    use solana_program::entrypoint;
    entrypoint!(process_instruction);
}

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("process instruction for printing sysvar");

    let account_info_iter = &mut accounts.iter();

    let payer_account = next_account_info(account_info_iter)?;
    let system_program_account = next_account_info(account_info_iter)?;

    let clock_account = next_account_info(account_info_iter)?;
    let epoch_schedule_account = next_account_info(account_info_iter)?;
    let rent_account = next_account_info(account_info_iter)?;

    {
        use solana_program::sysvar::clock;

        assert!(clock::check_id(clock_account.key));
        assert_eq!(*clock_account.key, clock::ID);

        let clock_from_account = clock::Clock::from_account_info(clock_account)?;
        let clock_from_sysvar = clock::Clock::get()?;

        assert_eq!(clock_from_account, clock_from_sysvar);

        msg!("clock: {:#?}", clock_from_account);
    }

    {
        use solana_program::sysvar::epoch_schedule;

        assert!(epoch_schedule::check_id(epoch_schedule_account.key));
        assert_eq!(*epoch_schedule_account.key, epoch_schedule::ID);

        let epoch_schedule_from_account =
            epoch_schedule::EpochSchedule::from_account_info(epoch_schedule_account)?;
        let epoch_schedule_from_sysvar = epoch_schedule::EpochSchedule::get()?;

        assert_eq!(epoch_schedule_from_account, epoch_schedule_from_sysvar);

        msg!("epoch_schedule: {:#?}", epoch_schedule_from_sysvar);
    }

    {
        use solana_program::sysvar::rent;

        assert!(rent::check_id(rent_account.key));
        assert_eq!(*rent_account.key, rent::ID);

        let rent_from_account = rent::Rent::from_account_info(rent_account)?;
        let rent_from_sysvar = rent::Rent::get()?;

        assert_eq!(rent_from_account, rent_from_sysvar);

        msg!("rent: {:#?}", rent_from_account);
    }

    Ok(())
}
