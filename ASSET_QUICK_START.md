# Asset Support System - Quick Start Guide

## For Developers

### What Was Implemented
A complete asset management system that allows the StellarAid contract to:
- Accept donations in multiple Stellar assets (XLM, USDC, NGNT, USDT, EURT)
- Allow admins to add/remove supported assets dynamically
- Validate assets before processing donations
- Store asset configuration on-chain

### Key Files Modified/Created

#### New Files:
- `crates/contracts/core/src/assets/storage.rs` - Asset configuration storage
- `.cargo/config.toml` - Build configuration
- `ASSET_IMPLEMENTATION_SUMMARY.md` - Detailed documentation

#### Modified Files:
- `crates/contracts/core/src/lib.rs` - Added admin functions and tests
- `crates/contracts/core/src/donation.rs` - Added asset validation
- `crates/contracts/core/src/validation/errors.rs` - Added UnsupportedAsset error
- `crates/contracts/core/src/assets/mod.rs` - Added storage module
- `crates/contracts/core/src/assets/resolver.rs` - Updated to use dynamic storage
- `crates/contracts/core/src/assets/validation.rs` - Updated validation signatures

### How to Use

#### As a User (Donor)
```rust
// Donate with XLM
let result = client.donate(
    &donor_address,
    &1000i128,  // Amount
    &String::from_str(&env, "XLM"),  // Asset
    &String::from_str(&env, "my-project"),  // Project ID
    &String::from_str(&env, "tx-hash-123")  // Transaction hash
);

// Only supported assets will succeed
// Unsupported assets return 0
```

#### As an Admin
```rust
// Initialize contract (sets you as admin)
client.init(&admin_address);

// Add a new supported asset
client.add_supported_asset(&admin, &String::from_str(&env, "BTC"));

// Remove an asset
client.remove_supported_asset(&admin, &String::from_str(&env, "EURT"));

// Check supported assets
let assets = client.get_supported_assets();

// Transfer admin rights
client.update_asset_admin(&admin, &new_admin);
```

### Default Supported Assets
When initialized, these 5 assets are supported:
1. **XLM** - Stellar Lumens (native, 7 decimals)
2. **USDC** - USD Coin (GA5Z..., 6 decimals)
3. **NGNT** - Nigerian Naira (GAUY..., 6 decimals)
4. **USDT** - Tether (GBBD..., 6 decimals)
5. **EURT** - Euro Token (GAP5..., 6 decimals)

### Testing
Run the test suite:
```bash
make test
```

Key tests to check:
- `test_donate_with_supported_asset_xlm` ✓
- `test_donate_with_unsupported_asset_rejected` ✓
- `test_admin_add_supported_asset` ✓
- `test_donation_with_all_supported_assets` ✓

### Error Codes
- **Error 15** (`UnsupportedAsset`) - Asset not in supported list
- **"Unauthorized"** - Caller is not admin
- **"Asset already supported"** - Trying to add existing asset
- **"Asset not in supported list"** - Trying to remove non-existent asset

## Architecture Overview

```
┌─────────────────────────────────────┐
│   User calls donate(asset)          │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│   validate_donation_with_error()    │
│   - Check amount > 0                │
│   - Check asset not empty           │
│   - Check asset supported ←────┐    │
│   - Check project_id valid      │    │
└──────────────┬──────────────────┼────┘
               │                  │
               ▼                  │
┌─────────────────────────────────┴────┐
│   AssetResolver::is_supported()      │
│   └─> AssetConfig::is_asset_supported()
│       └─> Checks on-chain storage    │
└──────────────────────────────────────┘
               │
               ├─✓ Supported → Process donation
               │
               └─✗ Unsupported → Return 0, emit error
```

## Security Features

### Access Control
- ✅ All admin functions check caller identity
- ✅ Only admin can add/remove assets
- ✅ Admin can be transferred securely

### Validation
- ✅ Assets validated before processing
- ✅ No reentrancy vulnerabilities
- ✅ Atomic state changes

### Storage
- ✅ On-chain storage for persistence
- ✅ Type-safe operations
- ✅ Efficient lookups

## Common Questions

**Q: Can I add custom assets?**  
A: Yes, the admin can add any asset using `add_supported_asset()`.

**Q: What happens if someone donates with unsupported asset?**  
A: The transaction returns 0 and the donation is rejected.

**Q: Can the admin remove all assets?**  
A: Technically yes, but not recommended as it would break donations.

**Q: How do I check which assets are supported?**  
A: Call `get_supported_assets()` to get the full list.

**Q: Can I update asset metadata?**  
A: Currently metadata is static. This could be added in future updates.

## Next Steps

1. **Build the contract** (requires proper toolchain):
   ```bash
   make wasm
   ```

2. **Deploy to testnet**:
   ```bash
   soroban contract deploy ...
   ```

3. **Initialize with admin**:
   ```bash
   soroban contract invoke -- init --admin <ADMIN_ADDRESS>
   ```

4. **Start accepting donations**:
   ```bash
   soroban contract invoke -- donate --donor ... --asset XLM ...
   ```

## Need Help?

- See `ASSET_IMPLEMENTATION_SUMMARY.md` for detailed documentation
- Check test examples in `lib.rs`
- Review architecture in `ARCHITECTURE.md`
