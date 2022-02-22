# Log


## 2022-02-22

`Unsupported sysvar` error when testing `slot_history`:

```rust
slot_history::SlotHistory::from_account_info(slot_history_account)?;
slot_history::SlotHistory::get()?;
```

Solana code:

> sdk/program/src/sysvar/slot_history.rs

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

