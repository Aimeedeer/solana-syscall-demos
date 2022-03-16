use borsh::de::BorshDeserialize;
use common::{
    system_test::{Allocate, CreateAccount, SystemTestInstruction, TransferLamports, TransferLamportsToMany},
    sysvar_test::SysvarTestInstruction,
    ProgramInstruction,
};
use solana_program::{
    account_info::{next_account_info, next_account_infos, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    pubkey::Pubkey,
    system_instruction, system_program,
    sysvar::rent::Rent,
    sysvar::Sysvar,
};

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("process instruction");

    let instr = ProgramInstruction::deserialize(&mut &instruction_data[..])?;

    let instr: &dyn Exec = match &instr {
        ProgramInstruction::SystemTest(instr) => match &instr {
            SystemTestInstruction::CreateAccount(instr) => instr,
            SystemTestInstruction::TransferLamports(instr) => instr,
            SystemTestInstruction::TransferLamportsToMany(instr) => instr,
            SystemTestInstruction::Allocate(instr) => instr,
        },
        ProgramInstruction::SysvarTest(instr) => instr,
    };
    instr.exec(program_id, accounts)
}

trait Exec {
    fn exec(&self, program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult;
}

impl Exec for CreateAccount {
    fn exec(&self, _program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        msg!("--------------------------------------- create_account_via_program ---------------------------------------");

        let account_info_iter = &mut accounts.iter();

        let payer = next_account_info(account_info_iter)?;
        let new_account = next_account_info(account_info_iter)?;
        let system_account = next_account_info(account_info_iter)?;

        {
            assert!(new_account.is_signer);
            assert_eq!(system_account.key, &system_program::ID);
        }

        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(
            self.space
                .try_into()
                .expect("failed converting `space` from u64 to usize"),
        );

        invoke(
            &system_instruction::create_account(
                payer.key,
                new_account.key,
                lamports,
                self.space,
                &system_program::ID,
            ),
            &[payer.clone(), new_account.clone(), system_account.clone()],
        )
    }
}

impl Exec for Allocate {
    fn exec(&self, _program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        msg!("--------------------------------------- allocate_via_program ---------------------------------------");

        let account_info_iter = &mut accounts.iter();

        let payer = next_account_info(account_info_iter)?;
        let new_account = next_account_info(account_info_iter)?;
        let system_account = next_account_info(account_info_iter)?;

        {
            assert!(new_account.is_signer);
            assert_eq!(system_account.key, &system_program::ID);
        }

        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(
            self.space
                .try_into()
                .expect("failed converting `space` from u64 to usize"),
        );

        invoke(
            &system_instruction::transfer(payer.key, new_account.key, lamports),
            &[payer.clone(), new_account.clone()],
        )?;

        invoke(
            &system_instruction::allocate(new_account.key, self.space),
            &[payer.clone(), new_account.clone(), system_account.clone()],
        )
    }
}

impl Exec for TransferLamports {
    fn exec(&self, _program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        msg!("--------------------------------------- transfer_via_program ---------------------------------------");

        let account_info_iter = &mut accounts.iter();

        let from = next_account_info(account_info_iter)?;
        let to = next_account_info(account_info_iter)?;
        let system_account = next_account_info(account_info_iter)?;

        assert_eq!(system_account.key, &system_program::ID);

        invoke(
            &system_instruction::transfer(from.key, to.key, self.amount),
            &[from.clone(), to.clone()],
        )
    }
}

impl Exec for TransferLamportsToMany {
    fn exec(&self, _program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        msg!("--------------------------------------- transfer_many_via_program ---------------------------------------");

        let account_info_iter = &mut accounts.iter();

        let from = next_account_info(account_info_iter)?;
        let system_account = next_account_info(account_info_iter)?;
        let to_accounts = next_account_infos(account_info_iter, account_info_iter.len())?;

        assert_eq!(system_account.key, &system_program::ID);

        let to_and_amount = to_accounts
            .iter()
            .zip(self.amount_list.iter())
            .map(|(to, amount)| (*to.key, *amount))
            .collect::<Vec<(Pubkey, u64)>>();

        let instr_list = system_instruction::transfer_many(from.key, to_and_amount.as_ref());

        for instr in instr_list {
            invoke(&instr, accounts)?;
        }

        Ok(())
    }
}

impl Exec for SysvarTestInstruction {
    fn exec(&self, _program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        msg!("--------------------------------------- sysvar program printing ---------------------------------------");

        let account_info_iter = &mut accounts.iter();

        let _payer_account = next_account_info(account_info_iter)?;
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

            let epoch_schedule_from_account =
                EpochSchedule::from_account_info(epoch_schedule_account)?;
            let epoch_schedule_from_sysvar = EpochSchedule::get()?;

            assert_eq!(epoch_schedule_from_account, epoch_schedule_from_sysvar);

            msg!("epoch_schedule: {:#?}", epoch_schedule_from_sysvar);
        }

        {
            use solana_program::sysvar::instructions;

            assert!(instructions::check_id(instructions_account.key));
            assert_eq!(*instructions_account.key, instructions::ID);

            let current_index = instructions::load_current_index_checked(instructions_account)?;
            let instructions_from_account = instructions::load_instruction_at_checked(
                current_index.into(),
                instructions_account,
            )?;

            let instr_from_account = instructions_from_account.data;
            let instr_from_account = ProgramInstruction::deserialize(&mut &instr_from_account[..])?;

            msg!(
                "deserialized instruction data from account: {:#?}",
                instr_from_account
            );
        }

        {
            use solana_program::sysvar::rent;

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
}
