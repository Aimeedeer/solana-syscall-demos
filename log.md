# Log

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

