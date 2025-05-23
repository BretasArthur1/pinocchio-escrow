# Solana Escrow Smart Contract

[![Solana](https://img.shields.io/badge/Solana-Blockchain-blueviolet)](https://solana.com/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## Overview

**Pinocchio Escrow** is a Solana smart contract (program) that implements a secure, token-based escrow system. It allows two parties to safely exchange tokens using an on-chain vault, ensuring trustless swaps without intermediaries.

## Features
- Trustless escrow for SPL tokens
- Secure vault using Program Derived Addresses (PDAs)
- Refund mechanism for makers
- Atomic swap: either both parties get what they want, or nothing happens
- Written in Rust for Solana

## How It Works
1. **Make**: The maker creates an escrow offer, locking their tokens in a vault and specifying what they want in return.
2. **Take**: A taker accepts the offer, sending the requested tokens. The program atomically swaps the tokens between the parties.
3. **Refund**: If no one takes the offer, the maker can reclaim their tokens.

## Account Structure
- **Maker**: The user creating the escrow offer
- **Taker**: The user accepting the offer
- **Vault**: Token account holding the maker's tokens (owned by a PDA)
- **Escrow**: Account storing trade parameters and state

## Instructions
- `Make`: Create a new escrow offer
- `Take`: Accept an offer and complete the swap
- `Refund`: Cancel an offer and reclaim tokens

## Usage

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor](https://www.anchor-lang.com/docs/installation) (if using Anchor for tests)

### Build
```sh
cargo build-bpf
```

### Test
```sh
cargo test-bpf
```

### Deploy
```sh
solana program deploy <path-to-program.so>
```

## Example Workflow
1. Maker calls `Make` to create an escrow offer.
2. Taker calls `Take` to accept and swap tokens.
3. If no taker, maker can call `Refund` to get tokens back.

## Resources
- [Solana Documentation](https://solana.com/docs)
- [Solana Escrow Example (PaulX)](https://paulx.dev/blog/2021/01/14/programming-on-solana-an-introduction/)
- [Anchor Book](https://www.anchor-lang.com/)

## License
MIT 