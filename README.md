# BIP39_Hunter
A high-performance, multi-threaded Bitcoin wallet scanner that checks BIP39 mnemonics against target addresses.

## Features ✨

- 🚀 **Blazing fast** - Utilizes all CPU cores with parallel processing  
- 🔐 **Multi-format support** - Checks Legacy (P2PKH), SegWit (P2WPKH), and Taproot (P2TR) addresses  
- 📊 **Real-time stats** - Displays scanning progress every 5 seconds  
- 📝 **Match logging** - Automatically saves found wallets to `found.txt`  
- 🌐 **Offline operation** - No network calls for maximum security  

## Installation ⚙️

1. Ensure you have [Rust installed](https://www.rust-lang.org/tools/install)
2. Clone this repository:
   ```bash
   git clone https://github.com/yourusername/bip39-hunter.git
   cd bip39-hunter

    Build in release mode:
    bash

    cargo build --release

## Usage 🖥️

Prepare a text file with target addresses (one per line)

    Run the scanner:
    bash
    cargo build --release

When prompted, enter the path to your target addresses file

Executable located in ./target/release/bip39-hunter

## Output 📂

    found.txt - Contains all matching wallets with:

        Mnemonic phrase

        Legacy address

        Bech32 address

        Taproot address

        WIF private key

## Technical Details ⚡
Address Derivation Paths

    m/44'/0'/0'/0/0 - BIP44 standard path

    Supports all major address types from single mnemonic

## Performance

    Generates and checks ~50,000 mnemonics/second on 16-core CPU

    Scales linearly with additional cores

## Safety Warning ⚠️

This tool:

    ❌ Does NOT connect to any network

    ❌ Does NOT transmit any data

    ❌ Should ONLY be used with addresses you legally own

## License 📜

MIT License - Use responsibly and legally
