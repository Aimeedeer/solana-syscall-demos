# Solana Sysvar Printing

This program prints Solana's [`sysvar`
accounts](https://docs.solana.com/developing/runtime-facilities/sysvars).
It can be used for understanding how `sysvar` works both on-chain and off-chain.


It has three parts:
- `program`: an on-chain program for reading and printing sysvar
- `client`:
  - call the on-chain program to execute printing
  - use RPC client to request `sysvar` and print it out
- `common`: a library that is shared with `program` and `client` for
  customized instructions

There is a log file recording my tests with `localhost` and `devnet`:
[log](log.md)

## How to use it

Install Solana cli following the official doc: [Command-line
Guide](https://docs.solana.com/cli).

Clone the repo:
```
$ git clone https://github.com/Aimeedeer/solana-sysvar-printing.git
$ cd solana-sysvar-printing
```

Build `program`:

```
$ cargo build-bpf

To deploy this program:
  $ solana program deploy <your_dir>/solana-sysvar-printing/target/deploy/program.so
The program address will default to this keypair (override with --program-id):
  <your_dir>/solana-sysvar-printing/target/deploy/program-keypair.json
```

Before deploying `program`, make config to connect to desired network.
If you choose `localhost`, you'll need to run `solana-test-validator` in another window.

```
$ solana config set --url localhost
```

Deploy `program`:

``` 
$ solana program deploy <your_dir>/solana-sysvar-printing/target/deploy/program.so
Program Id: <your_program_id>
```

The client program has several modes that demonstrate different capabilities.
To list them run

```
$ cargo run -- --help
```

Printing sysvars via client calls

```
$ cargo run -- print-sysvars-via-client

program id: <your_program_id>
sig: <your_tx_signature>
--------------------------------------- sysvar client printing ---------------------------------------
clock::ID: SysvarC1ock11111111111111111111111111111111
clock::check_id: true
clock::Clock::size_of: 40
clock account: Account {
    lamports: 1169280,
    data.len: 40,
    owner: Sysvar1111111111111111111111111111111111111,
    executable: false,
    rent_epoch: 0,
    data: 83e700000000000036c611620000000000000000000000000100000000000000e56c126200000000,
}
clock account data: Clock {
    slot: 59267,
    epoch_start_timestamp: 1645332022,
    epoch: 0,
    leader_schedule_epoch: 1,
    unix_timestamp: 1645374693,
}

...

stake_history::ID: SysvarStakeHistory1111111111111111111111111
stake_history::check_id: true
stake_history::StakeHistory::size_of: 16392
stake_history account: Account {
    lamports: 114979200,
    data.len: 16392,
    owner: Sysvar1111111111111111111111111111111111111,
    executable: false,
    rent_epoch: 0,
    data: 00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
}
stake_history account data: StakeHistory(
    [],
)
```

Printing sysvars via program CPI calls:

```
$ cargo run -- print-sysvars-via-program
```

To see the result of this command, as printed by the `program`,
run `solana logs` in another window:

```
$ solana logs
Streaming transaction logs. Confirmed commitment
Transaction executed in slot 59595:
  Signature: <your_tx_signature>
  Status: Ok
  Log Messages:
    Program <your_program_id> invoke [1]
    Program log: sysvar program printing
    Program log: clock: Clock {
    slot: 59595,
    epoch_start_timestamp: 1645332022,
    epoch: 0,
    leader_schedule_epoch: 1,
    unix_timestamp: 1645374930,
}

...

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
    Program <your_program_id> consumed 204526 of 1400000 compute units
    Program <your_program_id> success
```





