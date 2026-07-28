//! MilestoneX CLI entry point.
//!
//! Parses sub-commands for config, network, vault, asset, signing, response,
//! keymanager, keypair, deploy, invoke, and account operations.

use anyhow::{Result, Context};
use std::env;

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
        "deploy" => handle_deploy(),
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
    println!("  response <cmd>        - Process/validate/save signed wallet responses");
    println!("  invoke <method> [args] - Invoke a method on the deployed campaign contract");
    println!("                          Requires: CONTRACT_ID (or SOROBAN_CONTRACT_ID) and");
    println!("                          SOROBAN_SOURCE (or STELLAR_SECRET_KEY) env vars.");
    println!();
    println!("Stubs (no-op placeholders, do not rely on in production):");
    println!("  deploy                - Stub. Use `stellar contract deploy` or `make deploy-testnet`.");
    println!();
    println!("Deprecated (still functional via delegation, will be removed):");
    println!("  account create        - Deprecated alias for `keypair generate-master`.");
    println!("  account fund          - Deprecated alias for `keypair fund`.");
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

fn handle_deploy() -> Result<()> {
    println!("🚀 The 'deploy' command is a stub and is NOT yet implemented in this binary.");
    println!("💡 For real deployments use one of:");
    println!("     make deploy-testnet                  # uses scripts/deploy.sh + stellar contract deploy");
    println!("     bash scripts/deploy.sh testnet       # ditto, direct script invocation");
    println!("        (loads $SOROBAN_ADMIN_SECRET_KEY from .env, deploys the WASM at");
    println!("         target/wasm32v1-none/release/milestonex_core.wasm to testnet)");
    println!("     stellar contract deploy \\");
    println!("         --wasm target/wasm32v1-none/release/milestonex_core.wasm \\");
    println!("         --source \"$SOROBAN_ADMIN_SECRET_KEY\" --network testnet      # native fallback");
    println!("⚠️  Note: the deploy scripts currently ship the legacy `milestonex-core`");
    println!("    binary even though `milestonex-campaign` is canonical (see README).");
    println!("🔗 Tracked in: https://github.com/MillestoneX/MilestoneX-Contracts/issues/37");
    Ok(())
}

fn handle_invoke(args: &[String]) -> Result<()> {
    use std::process::Command;

    if args.is_empty() {
        println!("🔄 Invoke a method on the deployed MilestoneX campaign contract.");
        println!();
        println!("Usage: milestonex-cli invoke <method> [--arg <json_value>]...");
        println!();
        println!("Required environment variables:");
        println!("  CONTRACT_ID          — the deployed contract address (or set SOROBAN_CONTRACT_ID)");
        println!("  SOROBAN_SOURCE       — source keypair name / secret key (or set STELLAR_SECRET_KEY)");
        println!("  SOROBAN_NETWORK      — testnet | mainnet (default: testnet)");
        println!();
        println!("Examples:");
        println!("  CONTRACT_ID=CB7... SOROBAN_SOURCE=alice \\");
        println!("    milestonex-cli invoke version");
        println!();
        println!("  CONTRACT_ID=CB7... SOROBAN_SOURCE=alice \\");
        println!("    milestonex-cli invoke get_dashboard_metrics --arg '1'");
        println!();
        println!("Canonical contract methods (milestonex-campaign):");
        println!("  version                   — return contract version");
        println!("  get_dashboard_metrics <id>— aggregate metrics for a campaign");
        println!("  get_campaign_report <id>  — full campaign report");
        println!("  get_platform_summary      — platform-wide statistics");
        println!("  get_donation_count <id>   — number of donations for a campaign");
        println!("  get_donor_count <id>      — unique donor count");
        println!("  get_release_count <id>    — milestone release count");
        println!("  get_total_tx_count <id>   — total transaction count");
        return Ok(());
    }

    let method = &args[0];

    // Resolve contract ID — prefer explicit env var, then .milestonex_contract_id file.
    let contract_id = env::var("CONTRACT_ID")
        .or_else(|_| env::var("SOROBAN_CONTRACT_ID"))
        .or_else(|_| {
            std::fs::read_to_string(".milestonex_contract_id")
                .map(|s| s.trim().to_string())
                .map_err(|_| env::VarError::NotPresent)
        })
        .context(
            "CONTRACT_ID not set. Set the CONTRACT_ID env var, or run `make deploy-testnet` \
             first — it writes the deployed address to .milestonex_contract_id automatically.",
        )?;

    // Resolve source key — prefer SOROBAN_SOURCE, then STELLAR_SECRET_KEY.
    let source = env::var("SOROBAN_SOURCE")
        .or_else(|_| env::var("STELLAR_SECRET_KEY"))
        .context(
            "SOROBAN_SOURCE not set. Set SOROBAN_SOURCE (keypair name) or STELLAR_SECRET_KEY \
             (secret key) so stellar contract invoke can sign the transaction.",
        )?;

    let network = env::var("SOROBAN_NETWORK").unwrap_or_else(|_| "testnet".to_string());

    // --- Build the stellar contract invoke argument list ---
    // Positional args after the method name are passed as-is after `--`.
    // They can be raw JSON scalars (numbers, strings, booleans) or JSON objects.
    // stellar-cli already speaks JSON on the command line, so we pass them through.
    let method_args: Vec<String> = args[1..].to_vec();

    // Validate JSON args so we surface mistakes before shelling out.
    for (i, arg) in method_args.iter().enumerate() {
        // Skip flag-style args like "--arg" that precede the value.
        if arg.starts_with('-') {
            continue;
        }
        // Try to parse as JSON; fall back gracefully (stellar-cli handles plain strings too).
        if !arg.is_empty() {
            if let Err(_) = serde_json::from_str::<serde_json::Value>(arg) {
                // Not valid JSON — treat as a plain string literal (stellar-cli accepts this).
                // Only warn if it looks like the user meant JSON (starts with { or [).
                if arg.starts_with('{') || arg.starts_with('[') {
                    println!(
                        "⚠️  Argument {} '{}' looks like JSON but failed to parse. \
                         Passing as-is — stellar-cli may reject it.",
                        i + 1,
                        arg
                    );
                }
            }
        }
    }

    println!("🔄 Invoking '{}' on network: {}", method, network);
    println!("📝 Contract ID: {}", contract_id);
    println!("🔑 Source:      {}", source);
    if !method_args.is_empty() {
        println!("📥 Args:        {}", method_args.join(" "));
    }
    println!();

    // Build the command:
    //   stellar contract invoke \
    //     --id <CONTRACT_ID> \
    //     --source <SOURCE> \
    //     --network <NETWORK> \
    //     -- <method> [method_args...]
    let mut cmd = Command::new("stellar");
    cmd.arg("contract")
        .arg("invoke")
        .arg("--id")
        .arg(&contract_id)
        .arg("--source")
        .arg(&source)
        .arg("--network")
        .arg(&network)
        .arg("--")
        .arg(method);

    for arg in &method_args {
        cmd.arg(arg);
    }

    let output = cmd.output().context(
        "Failed to execute `stellar contract invoke`. \
         Is stellar-cli installed? Run: cargo install --locked stellar-cli --features opt",
    )?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("✅ Invocation successful!");
        if !stdout.trim().is_empty() {
            println!("📤 Result: {}", stdout.trim());
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("❌ Invocation failed (exit code: {})", output.status);
        if !stdout.trim().is_empty() {
            println!("stdout: {}", stdout.trim());
        }
        if !stderr.trim().is_empty() {
            println!("stderr: {}", stderr.trim());
        }
        anyhow::bail!("stellar contract invoke exited with error");
    }

    Ok(())
}

fn handle_account(args: &[String]) -> Result<()> {
    // Always show the deprecation banner — this namespace is a thin alias kept
    // only for backward compatibility with scripts written before Issue #31
    // introduced the `keypair` sub-command.
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  ⚠️  DEPRECATION WARNING                                      ║");
    println!("║  The 'account' sub-command is deprecated and will be removed ║");
    println!("║  in a future release.  Please migrate to the 'keypair'       ║");
    println!("║  namespace:                                                   ║");
    println!("║                                                               ║");
    println!("║    account create        →  keypair generate-master          ║");
    println!("║    account fund <a> <n>  →  keypair fund <a> <n>             ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    if args.is_empty() {
        println!("Usage: milestonex-cli account <create|fund> [args...]");
        println!();
        println!("Commands (deprecated — use 'keypair' instead):");
        println!("  create                         — alias for 'keypair generate-master'");
        println!("  fund <account_pub> <amount_xlm> — alias for 'keypair fund'");
        println!();
        println!("Migrate now:");
        println!("  milestonex-cli keypair generate-master");
        println!("  milestonex-cli keypair fund <account_public_key> <amount_xlm>");
        return Ok(());
    }

    // Transparently delegate to the canonical keypair handlers.
    match args[0].as_str() {
        "create" => {
            println!("🔄 Delegating 'account create' → 'keypair generate-master'");
            println!();
            handle_keypair(&["generate-master".to_string()])?;
        }
        "fund" => {
            if args.len() < 3 {
                println!("Usage: milestonex-cli account fund <account_public_key> <amount_xlm>");
                println!();
                println!("Preferred form:");
                println!("  milestonex-cli keypair fund <account_public_key> <amount_xlm>");
                return Ok(());
            }
            println!("🔄 Delegating 'account fund' → 'keypair fund'");
            println!();
            handle_keypair(&["fund".to_string(), args[1].clone(), args[2].clone()])?;
        }
        _ => {
            println!("❌ Unknown account sub-command: '{}'", args[0]);
            println!();
            println!("The 'account' namespace only aliases:");
            println!("  account create   →  keypair generate-master");
            println!("  account fund     →  keypair fund <account> <amount>");
            println!();
            println!("Run 'milestonex-cli keypair' for the full keypair command list.");
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

