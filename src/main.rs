use bip39::Mnemonic;
use bitcoin::{
    bip32::{DerivationPath, ExtendedPrivKey},
    key::{PrivateKey, TweakedPublicKey},
    secp256k1::{self, Secp256k1},
    Address, Network, PublicKey,
};
use colored::*;
use rand::{rngs::OsRng, RngCore};
use std::{
    collections::HashSet,
    fs::{File, OpenOptions},
    io::{BufRead, Write},
    path::Path,
    process,
    str::FromStr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

fn load_target_addresses<P: AsRef<Path>>(path: P) -> HashSet<String> {
    let file = File::open(path).unwrap_or_else(|e| {
        eprintln!("Failed to open target address file: {}", e);
        process::exit(1);
    });
    let reader = std::io::BufReader::new(file);
    reader
        .lines()
        .filter_map(Result::ok)
        .map(|line| line.trim().to_string())
        .collect()
}

fn derive_addresses(
    secp: &Secp256k1<secp256k1::All>,
    xpriv: &ExtendedPrivKey,
) -> (String, String, String, String) {
    let private_key = PrivateKey {
        inner: xpriv.private_key,
        network: Network::Bitcoin,
        compressed: true,
    };
    let wif = private_key.to_wif();

    let secp_pubkey = secp256k1::PublicKey::from_secret_key(secp, &private_key.inner);
    let pubkey = PublicKey {
        compressed: true,
        inner: secp_pubkey,
    };

    let legacy_addr = Address::p2pkh(&pubkey, Network::Bitcoin).to_string();
    let bech32_addr = Address::p2wpkh(&pubkey, Network::Bitcoin)
        .expect("Failed to create Bech32 address")
        .to_string();
    let (x_only, _parity) = pubkey.inner.x_only_public_key();
    let taproot_addr = Address::p2tr_tweaked(
        TweakedPublicKey::dangerous_assume_tweaked(x_only),
        Network::Bitcoin,
    )
    .to_string();

    (legacy_addr, bech32_addr, taproot_addr, wif)
}

fn main() {
    println!("{}", "===============================================".red());
    println!("{}", "BIP39 BRUTE-FORCE SCANNER".yellow());
    println!("{}", "2024 by MΔBUSΞ".truecolor(255, 165, 0));
	println!("{}", " ");
    println!("{}", "Legacy - Bech32 - Taproot");
    println!("{}", "===============================================".red());
    println!();

    
    println!("Enter path to target address file (one per line)...");
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    let target_path = input.trim();

    let target_addresses: Arc<HashSet<String>> = Arc::new(load_target_addresses(target_path));
    println!("✓ Loaded {} target addresses.", target_addresses.len());

    let secp = Secp256k1::new();
    let cores = num_cpus::get();
    println!("🧠 Starting scanner using all {} CPU cores...", cores);

    let total_scanned = Arc::new(AtomicUsize::new(0));
    let found_file = Arc::new(Mutex::new(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open("found.txt")
            .expect("Cannot open found.txt"),
    ));

    // Status thread
    {
        let total_scanned = Arc::clone(&total_scanned);
        thread::spawn(move || loop {
            let scanned = total_scanned.load(Ordering::Relaxed);
            println!("[STATUS] Total mnemonics scanned: {}", scanned);
            thread::sleep(Duration::from_secs(5));
        });
    }

    rayon::scope(|s| {
        for _ in 0..cores {
            let secp = secp.clone();
            let target_addresses = Arc::clone(&target_addresses);
            let total_scanned = Arc::clone(&total_scanned);
            let found_file = Arc::clone(&found_file);

            s.spawn(move |_| {
                let mut rng = OsRng;
                loop {
                    let mut entropy = [0u8; 16];
                    rng.fill_bytes(&mut entropy);

                    let mnemonic = Mnemonic::from_entropy(&entropy)
                        .expect("Failed to create mnemonic");

                    let seed = mnemonic.to_seed("");
                    let path =
                        DerivationPath::from_str("m/44'/0'/0'/0/0").expect("Invalid derivation path");
                    let xpriv = ExtendedPrivKey::new_master(Network::Bitcoin, &seed)
                        .expect("Failed to create master xpriv");
                    let child_xpriv = xpriv
                        .derive_priv(&secp, &path)
                        .expect("Failed to derive child xpriv");

                    let (legacy_addr, bech32_addr, taproot_addr, wif) =
                        derive_addresses(&secp, &child_xpriv);

                    total_scanned.fetch_add(1, Ordering::Relaxed);

                    if target_addresses.contains(&legacy_addr)
                        || target_addresses.contains(&bech32_addr)
                        || (!taproot_addr.is_empty() && target_addresses.contains(&taproot_addr))
                    {
                        let mut found = found_file.lock().unwrap();
                        writeln!(
                            found,
                            "MATCH FOUND!\nMnemonic: {}\nLegacy: {}\nBech32: {}\nTaproot: {}\nWIF: {}\n",
                            mnemonic,
                            legacy_addr,
                            bech32_addr,
                            taproot_addr,
                            wif
                        )
                        .expect("Failed to write to found.txt");
                        found.flush().unwrap();

                        println!("{}", "!!! MATCH FOUND !!!".green().bold());
                        println!("Mnemonic: {}", mnemonic);
                        println!("Legacy Address: {}", legacy_addr);
                        println!("Bech32 Address: {}", bech32_addr);
                        println!("Taproot Address: {}", taproot_addr);
                        println!("WIF: {}", wif);
                        println!("{}", "------------------------".blue());
                    }
                }
            });
        }
    });
}