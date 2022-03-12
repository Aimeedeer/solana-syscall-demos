# Solana Cheat Sheet (WIP)

**Need to be updated**

This program shows examples of using Solana's
[`system_instruction`][sysins] and [`sysvar` accounts][sysvar]. 
It can be used for understanding how `system_instruction` & `sysvar` work
both on-chain and off-chain.

[sysins]: https://docs.rs/solana-program/1.10.0/solana_program/system_instruction/index.html
[sysvar]: https://docs.solana.com/developing/runtime-facilities/sysvars

It has three parts:
- `program`: an on-chain program for executing `system_instruction`, reading and printing `sysvar`
- `client`:
  - call our deployed on-chain program to execute instructions
  - use RPC client to request `system_instruction` and `sysvar` directly from Solana node
- `common`: a library that is shared with `program` and `client` for
  customized instructions

There is a [log file](log.md) recording some of my tests with `localhost` and `devnet`.

## How to use it

Install Solana cli following the official doc: [Command-line Guide][solana-cli].

[solana-cli]: https://docs.solana.com/cli

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

**Things to be aware of:**

`devnet` runtime provides 200,000 compute units, while `localhost`
offers 1,400,000 compute units. (I didn't test it on `testnet` or
`mainnet-beta`.) 

This program works fine with `localhost`, but it
needs to be commented out some code in `components/program/src/lib.rs`
to avoid running out of computing budget while executing on `devnet`.

Run it and print results:

```
$ cargo run

program id: <your_program_id>
sig: <your_tx_signature>
create_account_via_program tx signature: 5g9prdnTVkLv7Qw84r4Q1oPA8hS3oNAZm45Xq8KuXWGbxw4dpiDyAqo1HXbGVkdqLW3ZSxxLi27pv8wAQft1zfq1
new_account_for_program_test created: Account {
    lamports: 890880,
    data.len: 0,
    owner: 11111111111111111111111111111111,
    executable: false,
    rent_epoch: 0,
}
allocate_via_program tx signature: 5C2jtiVoY1jeszbx392UHHJdkW8GZXHdd3kX7KuYm6JVSNfLBVSG4UWD5AjdLMhP4fD6EbymQo3iiYW8G9KPnA8E
account after allocate_via_program: Account {
    lamports: 8908800,
    data.len: 1024,
    owner: 11111111111111111111111111111111,
    executable: false,
    rent_epoch: 0,
    data: 00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
}
transfer_via_program tx signature: 3hsme1Dr126QqXy1XK14VLhRmFH6Q6qkN4GiNxHEFfE87Qc7RiT7MLY8xpYD7SotLuRBpPDibTHnHSxiCcSGfwMf
account after transfer_via_program: Account {
    lamports: 9908800,
    data.len: 1024,
    owner: 11111111111111111111111111111111,
    executable: false,
    rent_epoch: 0,
    data: 00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
}
--------------------------------------- create_account_via_rpc ---------------------------------------
create_account_via_rpc tx signature: WuRpXsswQANkAEPf1SAcs7SFw3H3CVTS2EQfYkNwQhZGuG8ucThxaZLNHe8a88NBCM1w9v7chT8f3cXwTLDgK2D
account before allocate_via_rpc: Account {
    lamports: 890880,
    data.len: 0,
    owner: 11111111111111111111111111111111,
    executable: false,
    rent_epoch: 0,
}
--------------------------------------- allocate_via_rpc ---------------------------------------
allocate_via_rpc tx signature: FZor9qbxVMhEZZ5YGBvLfXZPyVXkCDtWkSJZWtCRBGTnXJTkutZoes3xRMQ22bfmGF7kKXXMscqkyzZtFJ2EGUT
account after allocate_via_rpc: Account {
    lamports: 16035840,
    data.len: 2048,
    owner: 11111111111111111111111111111111,
    executable: false,
    rent_epoch: 0,
    data: 00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
}
--------------------------------------- transfer_via_rpc ---------------------------------------
transfer_via_rpc tx signature: 2c7Qoxz5b3FVjG7Tty19krRtpkPFd4Tm8mMwumSFrvQtTdTH3J84WbHEdWD2whaaAZVt55XiS1ZqU7Hz7xgFjxju
account after transfer_via_rpc: Account {
    lamports: 17035840,
    data.len: 2048,
    owner: 11111111111111111111111111111111,
    executable: false,
    rent_epoch: 0,
    data: 00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,
}
sysvar_printing_via_program sig: 5RxaegvwxQTqrPmhAN8dY3EUsdDuF6RXRosNdiyGBjRpvPFFKhPbMY2WgWWMe57V5qtCFsuh4cebK7kbYjm4hnyJ
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

To see `program`'s printing, run `solana logs` in another window:

```
Streaming transaction logs. Confirmed commitment
Transaction executed in slot 9370:
  Signature: 4uzEqyLZGyoJTgMPoGDyez49vTnc4gJFtJNpjfPfMH7rRFb4tRAjAf6BsmPqEju5Z6SBX3MF3brk8RudCZxwPwDA
  Status: Ok
  Log Messages:
    Program <your_program_id> invoke [1]
    Program log: process instruction
    Program log: --------------------------------------- create_account_via_program ---------------------------------------
    Program 11111111111111111111111111111111 invoke [2]
    Program 11111111111111111111111111111111 success
    Program <your_program_id> consumed 7000 of 1400000 compute units
    Program <your_program_id> success
Transaction executed in slot 9371:
  Signature: 2Buf6rGFbvkWEvGjJvcLWthh7akZomGs9nFeXwL6Y1GvjF6JrbBntHBr3anr12Y4yZGu8Lg2PNFmvfLjVgufvuSZ
  Status: Ok
  Log Messages:
    Program <your_program_id> invoke [1]
    Program log: process instruction
    Program log: --------------------------------------- allocate_via_program ---------------------------------------
    Program 11111111111111111111111111111111 invoke [2]
    Program 11111111111111111111111111111111 success
    Program 11111111111111111111111111111111 invoke [2]
    Program 11111111111111111111111111111111 success
    Program <your_program_id> consumed 6595 of 1400000 compute units
    Program <your_program_id> success
Transaction executed in slot 9372:
  Signature: wSQFm83h9hVKJhrg9WokFXWmgrJGi1qKzFAsyG2hnRG2xzCB7wb64d8TxzFaTPckaM5mzutp6v8L9ZkN8ZWLnCr
  Status: Ok
  Log Messages:
    Program <your_program_id> invoke [1]
    Program log: process instruction
    Program log: --------------------------------------- transfer_via_program ---------------------------------------
    Program 11111111111111111111111111111111 invoke [2]
    Program 11111111111111111111111111111111 success
    Program <your_program_id> consumed 3772 of 1400000 compute units
    Program <your_program_id> success
Transaction executed in slot 9373:
  Signature: 261DLPLDeVXNThrKFfuTqVXV7AWQksfLqX6Se28h4GurZcnCf4z2LSAj3eB3jnKR4FXZKt1KC4PmdL9boRgCjvgX
  Status: Ok
  Log Messages:
    Program 11111111111111111111111111111111 invoke [1]
    Program 11111111111111111111111111111111 success
Transaction executed in slot 9374:
  Signature: 5HR2wzcdL928HLQDEwJ6tsmbKedp9utVa3KnzJrhrN3735CtZRkGfBj7xsYNGwEPpZ9FTY2N8y4tLouzbSkroX7F
  Status: Ok
  Log Messages:
    Program 11111111111111111111111111111111 invoke [1]
    Program 11111111111111111111111111111111 success
    Program 11111111111111111111111111111111 invoke [1]
    Program 11111111111111111111111111111111 success
Transaction executed in slot 9375:
  Signature: 5tdeRaDw2NVoiWdY4qAkxWSuMF8rSvmo8P7s2DFjKExfntiAMs3xneWy5v1peLkiDFvTGe5hUkFSDwrRKbiLrJHB
  Status: Ok
  Log Messages:
    Program 11111111111111111111111111111111 invoke [1]
    Program 11111111111111111111111111111111 success
Transaction executed in slot 9377:
  Signature: 38wgREyJLKGw7yuzWF68VccsKSR7tmxTqzNWmzn22fmdtNEqTnt2Bbx3w5wKAVn8Dm2ddhRmmAUysBad9qiS5Xuy
  Status: Ok
  Log Messages:
    Program <your_program_id> invoke [1]
    Program log: process instruction
    Program log: --------------------------------------- sysvar program printing ---------------------------------------
    Program log: clock: Clock {
    slot: 9377,
    epoch_start_timestamp: 1647040592,
    epoch: 0,
    leader_schedule_epoch: 1,
    unix_timestamp: 1647047342,
}
    Program log: epoch_schedule: EpochSchedule {
    slots_per_epoch: 432000,
    leader_schedule_slot_offset: 432000,
    warmup: false,
    first_normal_epoch: 0,
    first_normal_slot: 0,
}
    Program log: deserialized instruction data from account: SysvarTest(
    SysvarTestInstruction {
        test_amount: 1000,
    },
)
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
    data: 0002000000000000a024000000000000d80f4da97e8f6b87a2dd2e055c07eeef5172da99d3f2839daa8528c2b1d6e9589f240000000000001723c8f17c881bc5,
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
    Program <your_program_id> consumed 206190 of 1400000 compute units
    Program <your_program_id> success
```







