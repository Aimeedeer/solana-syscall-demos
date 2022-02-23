# Log

## 2022-02-23

### Client errors

`Unsupported sysvar` error when calling `get()`:

```rust
println!("clock: {:#?}", clock::Clock::get());
```

## 2022-02-22

### Program errors

`Unsupported sysvar` error when testing `slot_history`, `slot_hashes` and `stake_history`:

```rust
slot_history::SlotHistory::from_account_info(slot_history_account)?;
slot_history::SlotHistory::get()?;
```

Solana code:

sdk/program/src/sysvar/slot_hashes.rs

```rust
impl Sysvar for SlotHashes {
    // override
    fn size_of() -> usize {
        // hard-coded so that we don't have to construct an empty
        20_488 // golden, update if MAX_ENTRIES changes
    }
    fn from_account_info(_account_info: &AccountInfo) -> Result<Self, ProgramError> {
        // This sysvar is too large to bincode::deserialize in-program
        Err(ProgramError::UnsupportedSysvar)
    }
}
```

sdk/program/src/sysvar/slot_history.rs

```rust
impl Sysvar for SlotHistory {
    // override
    fn size_of() -> usize {
        // hard-coded so that we don't have to construct an empty
        131_097 // golden, update if MAX_ENTRIES changes
    }
    fn from_account_info(_account_info: &AccountInfo) -> Result<Self, ProgramError> {
        // This sysvar is too large to bincode::deserialize in-program
        Err(ProgramError::UnsupportedSysvar)
    }
}
```
sdk/program/src/sysvar/stake_history.rs

```rust
impl Sysvar for StakeHistory {
    // override
    fn size_of() -> usize {
        // hard-coded so that we don't have to construct an empty
        16392 // golden, update if MAX_ENTRIES changes
    }
}
```

### On chain program printing:

```
  Log Messages:
    Program <program_id> invoke [1]
    Program log: process instruction for printing sysvar
    Program log: clock: Clock {
    slot: 30092,
    epoch_start_timestamp: 1645332022,
    epoch: 0,
    leader_schedule_epoch: 1,
    unix_timestamp: 1645353687,
}
    Program log: epoch_schedule: EpochSchedule {
    slots_per_epoch: 432000,
    leader_schedule_slot_offset: 432000,
    warmup: false,
    first_normal_epoch: 0,
    first_normal_slot: 0,
}
    Program log: deserialized_instruction_data: CustomInstruction {
    test_amount: 1000,
}
    Program log: rent: Rent {
    lamports_per_byte_year: 3480,
    exemption_threshold: 2.0,
    burn_percent: 50,
}
    Program log: slot_hashes: AccountInfo {
    key: SysvarS1otHashes111111111111111111111111111,
    owner: Sysvar1111111111111111111111111111111111111,
    is_signer: false,
    is_writable: false,
    executable: false,
    rent_epoch: 0,
    lamports: 143487360,
    data.len: 20488,
    data: 00020000000000008b750000000000009e2a1cb845cd9a6ae582efcc3e913798ae25d221873c7bf067203289828ceacd8a75000000000000a8022c9674287e31,
    ..
}
    Program log: slot_history: AccountInfo {
    key: SysvarS1otHistory11111111111111111111111111,
    owner: Sysvar1111111111111111111111111111111111111,
    is_signer: false,
    is_writable: false,
    executable: false,
    rent_epoch: 0,
    lamports: 913326000,
    data.len: 131097,
    data: 010040000000000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff,
    ..
}
    Program log: stake_history: AccountInfo {
    key: SysvarStakeHistory1111111111111111111111111,
    owner: Sysvar1111111111111111111111111111111111111,
    is_signer: false,
    is_writable: false,
    executable: false,
    rent_epoch: 0,
    lamports: 114979200,
    data.len: 16392,
    data: 00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
    ..
}
    Program <program_id> consumed 204487 of 1400000 compute units
    Program <program_id> success
```

## 2022-02-19

Deploy error:

```
$ solana program deploy /Users/aimeez/github/solana-sysvar/target/deploy/program.so
Error: ELF error: ELF error: Found writable section (.bss._ZN75_$LT$solana_program..sysvar..ALL_IDS$u20$as$u20$core..ops..deref..Deref$GT$5deref11__stability4LAZY17heac9787eef57c54aE) in ELF, read-write data not supported
```

Removed `sysvar::is_sysvar_id` from `program/src/lib.rs`, and deployed successfully.

```rust
assert!(sysvar::is_sysvar_id(clock_account.key));
```

