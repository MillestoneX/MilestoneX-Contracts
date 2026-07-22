//! MilestoneX CLI entry point.
//!
//! Parses sub-commands for config, network, vault, asset, signing, response,
//! keymanager, keypair, deploy, invoke, and account operations.

use anyhow::{Result, Context};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

mod environment_config;
use environment_config::{EnvironmentConfig, check_testnet_connection};

mod secure_vault;
use secure_vault::{SecureVault, check_mainnet_readiness, toggle_network};

mod asset_issuing;
use asset_issuing::{AssetConfig, check_issuing_readiness, generate_issuing_keypair, establish_trustline, issue_asset, TrustlineConfig};

mod key_manager;
use key_manager::KeyManager;

mod encrypted_vault;
use encrypted_vault::EncryptedVault;

mod keypair_manager;
use keypair_manager::{MasterKeypair, DistributionAccount, AccountFunding};

mod signing_request;
use signing_request::{SigningRequest, SigningRequestBuilder, TransactionBuilder};

mod response_handler;
use response_handler::{ResponseHandler, SignedTransaction};

fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_cli_banner();
        print_available_commands();
        return Ok(());
    }

    match args[1].as_str() {
        "config" => handle_config(),
        "network" => handle_network(),
        "vault" => handle_vault(),
        "toggle" => handle_toggle(&args[2..]),
        "asset" => handle_asset(&args[2..]),
        "deploy" => handle_deploy(&args[2..]),
        "invoke" => handle_invoke(&args[2..]),
        "account" => handle_account(&args[2..]),
        "keymanager" => handle_keymanager(&args[2..]),
        "keypair" => handle_keypair(&args[2..]),
        "signing" => handle_signing(&args[2..]),
        "response" => handle_response(&args[2..]),
        _ => {
            println!("❌ Unknown command: {}", args[1]);
            println!();
            print_available_commands();
            println!("🔗 See docs/deployment.md (Known Limitations) for unimplemented commands.");
            println!("   This gap is tracked in https://github.com/MillestoneX/MilestoneX-Contracts/issues/37");
            Ok(())
        }
    }
}

/// Print the MilestoneX CLI banner shown when no arguments are supplied,
/// or after an unknown command is requested.
fn print_cli_banner() {
    println!("MilestoneX CLI — Soroban Contract Management Tool");
    println!("Usage: milestonex-cli <command> [args...]");
}

/// Print every command currently wired into the dispatcher, grouped by area.
/// Stub commands are flagged so users do not assume they are production-ready.
///
/// Keep this in sync with `crates/tools/src/main.rs` `match args[1]` arms and
/// `docs/deployment.md` "Known Limitations / CLI Status".
fn print_available_commands() {
    println!("Implemented commands:");
    println!("  config                - Show resolved environment and network configuration");
    println!("  network               - Show active Soroban network (RPC, Horizon, passphrase)");
    println!("  vault                 - Show SecureVault status and security best practices");
    println!("  toggle <testnet|mainnet> - Switch the active network");
    println!("  asset <cmd>           - Asset issuing (config|generate|check|trustline|issue)");
    println!("  keymanager <cmd>      - Key encryption and encrypted vault lifecycle");
    println!("  keypair <cmd>         - Master/distribution keypair lifecycle");
    println!("  signing <cmd>         - Build donation/campaign/custom signing requests");
    println!("  deploy                - Deploy the canonical campaign WASM to the configured network (supports --dry-run, --source, --fee)");
    println!("  response <cmd>        - Process/validate/save signed wallet responses");
    println!();
    println!("Stubs (no-op placeholders, do not rely on in production):");
    println!("  invoke <method>       - Stub. Use `stellar contract invoke` natively.");
    println!();
    println!("Deprecated (still functional, but will be removed):");
    println!("  account <cmd>         - Deprecated. Use `keypair generate-master|fund` instead.");
    println!();
    println!("Run `milestonex-cli <command>` (no subcommand) for usage details.");
    println!("Full status of every command mentioned in docs: docs/deployment.md.");
}

fn handle_config() -> Result<()> {
    let config = EnvironmentConfig::from_env()?;
    
    println!("📋 Configuration Check");
    println!("━━━━━━━━━━━━━━━━━━━━━");
    println!("Active Network: {}", config.network);

    match config.network.as_str() {
        "testnet" => {
            println!("RPC URL: {}", config.testnet.rpc_url);
            println!("Horizon URL: {}", config.testnet.horizon_url);
            println!("Passphrase: {}", config.testnet.network_passphrase);
        }
        "mainnet" => {
            println!("RPC URL: {}", config.mainnet.rpc_url);
            println!("Horizon URL: {}", config.mainnet.horizon_url);
            println!("Passphrase: {}", config.mainnet.network_passphrase);
        }
        _ => println!("Unknown network: {}", config.network),
    }

    if let Some(ref admin_key) = config.admin_public_key {
        println!("Admin Public Key: {}", admin_key);
    } else {
        println!("⚠️  Admin public key not set");
    }

    // Validate configuration
    if let Err(e) = config.validate() {
        println!("❌ Configuration validation failed: {}", e);
    } else {
        println!("✅ Configuration is valid");
    }

    Ok(())
}

fn handle_network() -> Result<()> {
    let config = EnvironmentConfig::from_env()?;
    
    println!("🌐 Network Configuration");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Active Network: {}", config.network);

    match config.network.as_str() {
        "testnet" => {
            println!("RPC URL: {}", config.testnet.rpc_url);
            println!("Horizon URL: {}", config.testnet.horizon_url);
            println!("Passphrase: {}", config.testnet.network_passphrase);
        }
        "mainnet" => {
            println!("RPC URL: {}", config.mainnet.rpc_url);
            println!("Horizon URL: {}", config.mainnet.horizon_url);
            println!("Passphrase: {}", config.mainnet.network_passphrase);
        }
        _ => println!("Unknown network configuration"),
    }

    Ok(())
}

fn handle_deploy(args: &[String]) -> Result<()> {
    let mut dry_run = false;
    let mut source_override: Option<String> = None;
    let mut fee_override: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dry-run" => dry_run = true,
            "--source" => {
                i += 1;
                source_override = Some(
                    args.get(i)
                        .cloned()
                        .context("--source requires a value (key name or secret key)")?,
                );
            }
            "--fee" => {
                i += 1;
                fee_override = Some(
                    args.get(i)
                        .cloned()
                        .context("--fee requires a numeric value")?,
                );
            }
            other => anyhow::bail!("Unknown deploy flag: {other}"),
        }
        i += 1;
    }

    let network = env::var("SOROBAN_NETWORK").unwrap_or_else(|_| "testnet".to_string());

    let source = match source_override {
        Some(ref key_name) => key_name.clone(),
        None => env::var("SOROBAN_ADMIN_SECRET_KEY")
            .context("SOROBAN_ADMIN_SECRET_KEY not set. Provide --source <key_name> or set the env var")?,
    };

    let config = EnvironmentConfig::from_env()?;
    let active = config.get_active_network()?;

    let rpc_url = env::var("SOROBAN_RPC_URL").unwrap_or(active.rpc_url);
    let passphrase = env::var("SOROBAN_NETWORK_PASSPHRASE").unwrap_or(active.network_passphrase);

    let wasm_dir = "target/wasm32v1-none/release";
    let canonical_path = format!("{wasm_dir}/milestonex_campaign.wasm");
    let legacy_path = format!("{wasm_dir}/milestonex_core.wasm");

    let wasm_path = if Path::new(&canonical_path).exists() {
        canonical_path
    } else if Path::new(&legacy_path).exists() {
        legacy_path
    } else {
        anyhow::bail!(
            "WASM not found at {canonical_path} or {legacy_path} — run 'make build-wasm' first"
        );
    };

    let deployments_dir = "deployments";
    let deployment_file = format!("{deployments_dir}/{network}.json");

    if Path::new(&deployment_file).exists() {
        let content = fs::read_to_string(&deployment_file)?;
        if let Ok(record) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(id) = record.get("contract_id").and_then(|v| v.as_str()) {
                if !id.is_empty() {
                    println!("ℹ️  Contract already deployed on {network}: {id}");
                    println!("   Delete {deployment_file} to force a re-deploy.");
                    return Ok(());
                }
            }
        }
    }

    if dry_run {
        println!("🚀 Dry-run: would deploy to {network}");
        println!("   WASM: {wasm_path}");
        println!("   Source: {source}");
        println!("   RPC: {rpc_url}");
        if let Some(ref fee) = fee_override {
            println!("   Fee: {fee}");
        }
        return Ok(());
    }

    let mut cmd = Command::new("stellar");
    cmd.arg("contract")
        .arg("deploy")
        .arg("--wasm")
        .arg(&wasm_path)
        .arg("--source")
        .arg(&source)
        .arg("--rpc-url")
        .arg(&rpc_url)
        .arg("--network-passphrase")
        .arg(&passphrase);

    if let Some(ref fee) = fee_override {
        cmd.arg("--fee").arg(fee);
    }

    println!("🚀 Deploying to {network}...");
    println!("   RPC: {rpc_url}");
    println!("   WASM: {wasm_path}");

    let output = cmd
        .output()
        .context("Failed to execute 'stellar contract deploy'. Is stellar-cli installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Deployment failed:\n{stderr}");
    }

    let contract_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    println!("✅ Contract deployed!");
    println!("📝 Contract ID: {contract_id}");

    fs::create_dir_all(deployments_dir)?;
    let timestamp = chrono::Utc::now().to_rfc3339();
    let record = serde_json::json!({
        "network": network,
        "contract_id": contract_id,
        "rpc_url": rpc_url,
        "deployed_at": timestamp,
        "wasm": wasm_path,
    });

    let json = serde_json::to_string_pretty(&record)?;
    fs::write(&deployment_file, &json)?;
    println!("💾 Deployment record saved to {deployment_file}");

    fs::write(".milestonex_contract_id", &contract_id)?;
    println!("✅ Contract ID stored in .milestonex_contract_id");

    Ok(())
}

fn handle_invoke(args: &[String]) -> Result<()> {
    println!("🔄 The 'invoke' command is a stub and is NOT yet implemented in this binary.");
    if args.is_empty() {
        println!("💡 Invoke natively with:");
        println!("     stellar contract invoke \\");
        println!("         --id <CONTRACT_ID> \\");
        println!("         --source <KEY> \\");
        println!("         --network testnet \\");
        println!("         -- <method> [args...]");
    } else {
        println!("💡 You asked to invoke '{}'. Run it natively for now:", args[0]);
        println!("     stellar contract invoke \\");
        println!("         --id <CONTRACT_ID> \\");
        println!("         --source <KEY> \\");
        println!("         --network testnet \\");
        println!("         -- {} [args...]", args[0]);
    }
    println!("🔗 Tracked in: https://github.com/MillestoneX/MilestoneX-Contracts/issues/37");
    Ok(())
}

fn handle_account(args: &[String]) -> Result<()> {
    println!("⚠️  DEPRECATION WARNING: The 'account' namespace is deprecated and will be removed in a future release.");
    println!("💡 Please use the 'keypair' namespace instead. See below for mapping.");
    println!();

    if args.is_empty() {
        println!("👤 Account Management Commands (Deprecated)");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Usage: milestonex-cli account <command>");
        println!();
        println!("Commands (deprecated - use 'keypair' instead):");
        println!("  create                              - Create master keypair (use 'keypair generate-master')");
        println!("  fund <account> <amount>              - Fund account on testnet (use 'keypair fund')");
        println!();
        println!("💡 Migrate to:");
        println!("     milestonex-cli keypair generate-master");
        println!("     milestonex-cli keypair fund <account> <amount>");
        return Ok(());
    }

    // Map account commands to keypair commands for backward compatibility
    match args[0].as_str() {
        "create" => {
            println!("🔄 'account create' is deprecated. Delegating to 'keypair generate-master'...");
            println!();
            handle_keypair(&["generate-master".to_string()])?;
        }
        "fund" => {
            if args.len() < 3 {
                println!("Usage: milestonex-cli account fund <account_public_key> <amount_xlm>");
                println!("💡 Please use: milestonex-cli keypair fund <account_public_key> <amount_xlm>");
                return Ok(());
            }
            println!("🔄 'account fund' is deprecated. Delegating to 'keypair fund'...");
            println!();
            handle_keypair(&["fund".to_string(), args[1].clone(), args[2].clone()])?;
        }
        _ => {
            println!("❌ Unknown account command: {}", args[0]);
            println!("💡 The 'account' namespace is deprecated. Available commands:");
            println!("   account create   (use 'keypair generate-master')");
            println!("   account fund     (use 'keypair fund')");
            println!();
            println!("🔗 See https://github.com/MillestoneX/MilestoneX-Contracts/issues/37 for details");
        }
    }

    Ok(())
}

fn handle_vault() -> Result<()> {
    let vault = SecureVault::from_env();
    vault.display_safe();
    
    println!();
    println!("💡 Security Best Practices:");
    println!("   - Never commit secret keys to version control");
    println!("   - Use .env files and add them to .gitignore");
    println!("   - Rotate keys regularly");
    println!("   - Use separate keys for testnet and mainnet");
    
    Ok(())
}

fn handle_toggle(args: &[String]) -> Result<()> {
    if args.is_empty() {
        println!("Usage: milestonex-cli toggle <testnet|mainnet>");
        return Ok(());
    }

    toggle_network(args[0].as_str())
}

fn handle_asset(args: &[String]) -> Result<()> {
    if args.is_empty() {
        println!("🪙 Asset Management Commands");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Usage: milestonex-cli asset <command>");
        println!();
        println!("Commands:");
        println!("  config     - Show asset configuration");
        println!("  generate   - Generate issuing keypair");
        println!("  check      - Check issuing readiness");
        println!("  trustline  - Establish trustline");
        println!("  issue      - Issue assets to recipient");
        return Ok(());
    }

    match args[0].as_str() {
        "config" => {
            let config = AssetConfig::from_env()?;
            config.display();
        }
        "generate" => {
            generate_issuing_keypair()?;
        }
        "check" => {
            check_issuing_readiness()?;
        }
        "trustline" => {
            if args.len() < 3 {
                println!("Usage: milestonex-cli asset trustline <holder_public_key> [asset_code]");
                return Ok(());
            }
            
            let holder = &args[1];
            let asset_config = AssetConfig::from_env()?;
            let asset_code = if args.len() > 2 {
                args[2].clone()
            } else {
                asset_config.code.clone()
            };
            
            let network = env::var("SOROBAN_NETWORK").unwrap_or_else(|_| "testnet".to_string());
            
            let trustline_config = TrustlineConfig {
                asset_code,
                asset_issuer: asset_config.issuing_public_key,
                holder_public_key: holder.clone(),
            };
            
            establish_trustline(&trustline_config, &network)?;
        }
        "issue" => {
            if args.len() < 3 {
                println!("Usage: milestonex-cli asset issue <recipient> <amount>");
                return Ok(());
            }
            
            let recipient = &args[1];
            let amount: f64 = args[2].parse().context("Invalid amount")?;
            let network = env::var("SOROBAN_NETWORK").unwrap_or_else(|_| "testnet".to_string());
            let asset_config = AssetConfig::from_env()?;
            
            issue_asset(&asset_config, recipient, amount, &network)?;
        }
        _ => {
            println!("Unknown asset command: {}", args[0]);
            handle_asset(&[])?;
        }
    }

    Ok(())
}

fn handle_keymanager(args: &[String]) -> Result<()> {
    if args.is_empty() {
        println!("🔑 Key Manager Commands");
        println!("━━━━━━━━━━━━━━━━━━━━━━");
        println!("Usage: milestonex-cli keymanager <command>");
        println!();
        println!("Commands:");
        println!("  encrypt <password> <secret_key>  - Encrypt a secret key");
        println!("  decrypt <password> <encrypted>   - Decrypt an encrypted key");
        println!("  init-vault <password>            - Initialize encrypted vault");
        println!("  vault-status                     - Show vault status");
        println!("  vault-save <path>                - Save vault to file");
        println!("  vault-load <path> <password>     - Load vault from file");
        return Ok(());
    }

    match args[0].as_str() {
        "encrypt" => {
            if args.len() < 3 {
                println!("Usage: milestonex-cli keymanager encrypt <password> <secret_key>");
                return Ok(());
            }
            
            let password = &args[1];
            let secret_key = &args[2];
            
            KeyManager::validate_secret_key(secret_key)?;
            let manager = KeyManager::from_password(password)?;
            let encrypted_hex = manager.export_encrypted(secret_key)?;
            
            println!("✅ Key encrypted successfully");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("Encrypted Key (hex format):");
            println!("{}", encrypted_hex);
            println!();
            println!("💡 Store this encrypted key safely and use VAULT_MASTER_PASSWORD to decrypt");
        }
        "decrypt" => {
            if args.len() < 3 {
                println!("Usage: milestonex-cli keymanager decrypt <password> <encrypted_hex>");
                return Ok(());
            }
            
            let password = &args[1];
            let encrypted_hex = &args[2];
            
            let manager = KeyManager::from_password(password)?;
            let encrypted = manager.import_encrypted(encrypted_hex)?;
            let secret_key = manager.decrypt_key(&encrypted)?;
            
            println!("✅ Key decrypted successfully");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("Secret Key: {}", secret_key);
            println!();
            println!("⚠️  WARNING: Keep this secret key secure!");
        }
        "init-vault" => {
            if args.len() < 2 {
                println!("Usage: milestonex-cli keymanager init-vault <password>");
                return Ok(());
            }
            
            let password = &args[1];
            let mut vault = EncryptedVault::with_password(password)?;
            
            println!("✅ Encrypted vault initialized");
            vault.display_status();
            println!();
            println!("💡 Set VAULT_MASTER_PASSWORD={} in your .env file", password);
        }
        "vault-status" => {
            let vault = EncryptedVault::from_env()?;
            vault.display_status();
        }
        "vault-save" => {
            if args.len() < 2 {
                println!("Usage: milestonex-cli keymanager vault-save <path>");
                return Ok(());
            }
            
            let path = &args[1];
            let vault = EncryptedVault::from_env()?;
            vault.save_to_file(path)?;
        }
        "vault-load" => {
            if args.len() < 3 {
                println!("Usage: milestonex-cli keymanager vault-load <path> <password>");
                return Ok(());
            }
            
            let path = &args[1];
            let password = &args[2];
            
            let vault = EncryptedVault::load_from_file(path, password)?;
            vault.display_status();
        }
        _ => {
            println!("Unknown keymanager command: {}", args[0]);
            handle_keymanager(&[])?;
        }
    }

    Ok(())
}

fn handle_keypair(args: &[String]) -> Result<()> {
    if args.is_empty() {
        println!("🔑 Keypair Management Commands");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Usage: milestonex-cli keypair <command>");
        println!();
        println!("Commands:");
        println!("  generate-master                      - Generate master keypair");
        println!("  generate-distribution <issuing_pub>  - Generate distribution account");
        println!("  show-master                          - Show master keypair");
        println!("  show-distribution                    - Show distribution account");
        println!("  fund <account> <amount>              - Fund account on testnet");
        println!("  validate-master                      - Validate master keypair");
        println!("  validate-distribution                - Validate distribution account");
        return Ok(());
    }

    match args[0].as_str() {
        "generate-master" => {
            let network = env::var("SOROBAN_NETWORK").unwrap_or_else(|_| "testnet".to_string());
            
            println!("🔑 Generating Master Keypair");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            
            let keypair = MasterKeypair::generate(&network)?;
            keypair.display_safe();
            
            println!();
            println!("💡 Store this keypair securely:");
            println!("   milestonex-cli keymanager encrypt '<password>' '{}'", keypair.secret_key);
        }
        "generate-distribution" => {
            if args.len() < 2 {
                println!("Usage: milestonex-cli keypair generate-distribution <issuing_public_key>");
                return Ok(());
            }
            
            let issuing_pub = &args[1];
            let network = env::var("SOROBAN_NETWORK").unwrap_or_else(|_| "testnet".to_string());
            
            println!("💰 Generating Distribution Account");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            
            let dist = DistributionAccount::generate(&network, issuing_pub)?;
            dist.display_safe();
            
            println!();
            println!("💡 Link this distribution account to your issuing account");
        }
        "show-master" => {
            let vault = EncryptedVault::from_env()?;
            match MasterKeypair::load_from_vault(&vault) {
                Ok(keypair) => {
                    keypair.display_safe();
                }
                Err(_) => {
                    println!("❌ Master keypair not found in vault");
                    println!("💡 Generate it with: milestonex-cli keypair generate-master");
                }
            }
        }
        "show-distribution" => {
            let vault = EncryptedVault::from_env()?;
            match DistributionAccount::load_from_vault(&vault) {
                Ok(dist) => {
                    dist.display_safe();
                }
                Err(_) => {
                    println!("❌ Distribution account not found in vault");
                    println!("💡 Generate it with: milestonex-cli keypair generate-distribution <issuing_key>");
                }
            }
        }
        "fund" => {
            if args.len() < 3 {
                println!("Usage: milestonex-cli keypair fund <account_public_key> <amount_xlm>");
                return Ok(());
            }
            
            let account_pub = &args[1];
            let amount: f64 = args[2].parse().context("Invalid amount")?;
            let network = env::var("SOROBAN_NETWORK").unwrap_or_else(|_| "testnet".to_string());
            
            let mut funding = AccountFunding::new(account_pub, &network)?;
            funding.fund_testnet(amount)?;
            funding.display_status();
        }
        "validate-master" => {
            let vault = EncryptedVault::from_env()?;
            match MasterKeypair::load_from_vault(&vault) {
                Ok(keypair) => {
                    match keypair.validate() {
                        Ok(_) => {
                            println!("✅ Master keypair is valid");
                            keypair.display_safe();
                        }
                        Err(e) => {
                            println!("❌ Master keypair validation failed: {}", e);
                        }
                    }
                }
                Err(_) => {
                    println!("❌ Master keypair not found in vault");
                }
            }
        }
        "validate-distribution" => {
            let vault = EncryptedVault::from_env()?;
            match DistributionAccount::load_from_vault(&vault) {
                Ok(dist) => {
                    match dist.validate() {
                        Ok(_) => {
                            println!("✅ Distribution account is valid");
                            dist.display_safe();
                        }
                        Err(e) => {
                            println!("❌ Distribution account validation failed: {}", e);
                        }
                    }
                }
                Err(_) => {
                    println!("❌ Distribution account not found in vault");
                }
            }
        }
        _ => {
            println!("Unknown keypair command: {}", args[0]);
            handle_keypair(&[])?;
        }
    }

    Ok(())
}

fn handle_signing(args: &[String]) -> Result<()> {
    if args.is_empty() {
        println!("🔐 Signing Request Commands");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Usage: milestonex-cli signing <command>");
        println!();
        println!("Commands:");
        println!("  build-donation     - Build donation signing request");
        println!("  build-campaign     - Build campaign creation request");
        println!("  build-custom       - Build custom signing request");
        println!("  validate           - Validate signing request");
        println!("  export             - Export signing request to JSON");
        return Ok(());
    }

    match args[0].as_str() {
        "build-donation" => {
            if args.len() < 4 {
                println!("Usage: milestonex-cli signing build-donation <donor_address> <campaign_id> <amount> [asset] [memo]");
                return Ok(());
            }

            let donor = args[1].clone();
            let campaign_id: u64 = args[2].parse()
                .context("Invalid campaign ID")?;
            let amount: i128 = args[3].parse()
                .context("Invalid amount")?;
            let asset = if args.len() > 4 {
                args[4].clone()
            } else {
                "XLM".to_string()
            };
            let memo = if args.len() > 5 {
                Some(args[5].clone())
            } else {
                None
            };

            match TransactionBuilder::build_donation_request(donor, campaign_id, amount, asset, memo) {
                Ok(req) => {
                    req.display();
                    println!();
                    println!("💡 To submit to wallet:");
                    if let Ok(json) = req.to_json() {
                        println!("JSON: {}", json);
                    }
                }
                Err(e) => {
                    println!("❌ Failed to build donation request: {}", e);
                }
            }
        }
        "build-campaign" => {
            if args.len() < 4 {
                println!("Usage: milestonex-cli signing build-campaign <creator_address> <title> <goal> <deadline_timestamp>");
                return Ok(());
            }

            let creator = args[1].clone();
            let title = args[2].clone();
            let goal: i128 = args[3].parse()
                .context("Invalid goal")?;
            let deadline: u64 = args[4].parse()
                .context("Invalid deadline")?;

            match TransactionBuilder::build_campaign_request(creator, title, goal, deadline) {
                Ok(req) => {
                    req.display();
                    println!();
                    println!("💡 To submit to wallet:");
                    if let Ok(json) = req.to_json() {
                        println!("JSON: {}", json);
                    }
                }
                Err(e) => {
                    println!("❌ Failed to build campaign request: {}", e);
                }
            }
        }
        "build-custom" => {
            if args.len() < 2 {
                println!("Usage: milestonex-cli signing build-custom <xdr> [description]");
                return Ok(());
            }

            let xdr = args[1].clone();
            let description = if args.len() > 2 {
                args[2].clone()
            } else {
                "Custom transaction".to_string()
            };

            match SigningRequestBuilder::new(xdr, None) {
                Ok(builder) => {
                    match builder.with_description(description).build() {
                        Ok(req) => {
                            req.display();
                            println!();
                            println!("✅ Signing request created successfully");
                        }
                        Err(e) => {
                            println!("❌ Failed to build request: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to create builder: {}", e);
                }
            }
        }
        "validate" => {
            if args.len() < 2 {
                println!("Usage: milestonex-cli signing validate <json_file>");
                return Ok(());
            }

            let path = &args[1];
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    match SigningRequest::from_json(&content) {
                        Ok(req) => {
                            match req.validate() {
                                Ok(_) => {
                                    println!("✅ Signing request is valid");
                                    req.display();
                                }
                                Err(e) => {
                                    println!("❌ Validation failed: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("❌ Failed to parse request: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to read file: {}", e);
                }
            }
        }
        "export" => {
            if args.len() < 2 {
                println!("Usage: milestonex-cli signing export <json_file>");
                println!();
                println!("Exports a signing request in wallet-compatible format");
                return Ok(());
            }

            let path = &args[1];
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    match SigningRequest::from_json(&content) {
                        Ok(req) => {
                            match req.to_wallet_format() {
                                Ok(wallet_format) => {
                                    println!("📤 Wallet Format:");
                                    println!("{}", wallet_format);
                                }
                                Err(e) => {
                                    println!("❌ Failed to export: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("❌ Failed to parse request: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to read file: {}", e);
                }
            }
        }
        _ => {
            println!("Unknown signing command: {}", args[0]);
            handle_signing(&[])?;
        }
    }

    Ok(())
}

fn handle_response(args: &[String]) -> Result<()> {
    if args.is_empty() {
        println!("✅ Response Handler Commands");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Usage: milestonex-cli response <command>");
        println!();
        println!("Commands:");
        println!("  process       - Process wallet response JSON");
        println!("  validate      - Validate signed transaction");
        println!("  save          - Save signed transaction to file");
        println!("  load          - Load signed transaction from file");
        println!("  submit        - Submit signed transaction (placeholder)");
        return Ok(());
    }

    match args[0].as_str() {
        "process" => {
            if args.len() < 2 {
                println!("Usage: milestonex-cli response process <json_response>");
                return Ok(());
            }

            let response = args[1].clone();
            match ResponseHandler::process_response(&response) {
                Ok(processed) => {
                    processed.display();
                    println!();
                    if processed.is_valid() {
                        println!("Ready for submission");
                    }
                }
                Err(e) => {
                    println!("❌ Failed to process response: {}", e);
                }
            }
        }
        "validate" => {
            if args.len() < 2 {
                println!("Usage: milestonex-cli response validate <json_file>");
                return Ok(());
            }

            let path = &args[1];
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    match ResponseHandler::parse_response(&content) {
                        Ok(tx) => {
                            match ResponseHandler::validate(&tx) {
                                Ok(_) => {
                                    println!("✅ Transaction is valid");
                                    println!("Request ID:    {}", tx.request_id);
                                    println!("Signer:        {}", tx.signer);
                                    println!("Status:        {}", tx.status);
                                    println!("XDR Length:    {} bytes", tx.transaction_xdr.len());
                                }
                                Err(e) => {
                                    println!("❌ Validation failed: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("❌ Failed to parse response: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to read file: {}", e);
                }
            }
        }
        "save" => {
            if args.len() < 3 {
                println!("Usage: milestonex-cli response save <json_response> <output_file>");
                return Ok(());
            }

            let response = args[1].clone();
            let output_path = &args[2];

            match ResponseHandler::parse_response(&response) {
                Ok(tx) => {
                    match ResponseHandler::save_to_file(&tx, output_path) {
                        Ok(_) => {
                            println!("✅ Transaction saved to {}", output_path);
                            println!("Request ID: {}", tx.request_id);
                        }
                        Err(e) => {
                            println!("❌ Failed to save transaction: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to parse response: {}", e);
                }
            }
        }
        "load" => {
            if args.len() < 2 {
                println!("Usage: milestonex-cli response load <json_file>");
                return Ok(());
            }

            let path = &args[1];
            match ResponseHandler::load_from_file(path) {
                Ok(tx) => {
                    println!("✅ Transaction loaded from {}", path);
                    println!();
                    println!("Request ID:    {}", tx.request_id);
                    println!("Signer:        {}", tx.signer);
                    println!("Status:        {}", tx.status);
                    println!("Signed At:     {}", tx.signed_at);
                    println!();
                    println!("Transaction XDR:");
                    println!("{}", tx.transaction_xdr);
                }
                Err(e) => {
                    println!("❌ Failed to load transaction: {}", e);
                }
            }
        }
        "submit" => {
            if args.len() < 2 {
                println!("Usage: milestonex-cli response submit <json_file>");
                return Ok(());
            }

            let path = &args[1];
            match ResponseHandler::load_from_file(path) {
                Ok(tx) => {
                    println!("📤 Submitting Transaction");
                    println!("━━━━━━━━━━━━━━━━━━━━━━━");
                    println!("Request ID: {}", tx.request_id);
                    println!("Signer:     {}", tx.signer);
                    println!();
                    println!("🔄 Sending to Stellar network...");
                    println!();
                    println!("💡 Full submission implementation coming soon");
                    println!("   This would submit the signed transaction to:");
                    println!("   - Validate transaction format");
                    println!("   - Check sequence numbers");
                    println!("   - Post to Stellar network");
                    println!("   - Monitor for confirmation");
                }
                Err(e) => {
                    println!("❌ Failed to load transaction: {}", e);
                }
            }
        }
        _ => {
            println!("Unknown response command: {}", args[0]);
            handle_response(&[])?;
        }
    }

    Ok(())
}

