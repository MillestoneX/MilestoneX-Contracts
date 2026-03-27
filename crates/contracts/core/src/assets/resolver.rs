//! Asset Resolution Utilities
//!
//! Provides utilities for resolving and validating Stellar assets.

use soroban_sdk::{Env, String, Vec};

use super::config::{AssetRegistry, StellarAsset};
use super::metadata::MetadataRegistry;
use super::storage::AssetConfig;

/// Asset resolver for looking up and validating assets
pub struct AssetResolver;

impl AssetResolver {
    /// Resolve an asset by its code
    ///
    /// Returns the asset if found, otherwise None
    pub fn resolve_by_code(env: &Env, code: &str) -> Option<StellarAsset> {
        // First check if asset is supported via storage
        if !AssetConfig::is_asset_supported(env, code) {
            return None;
        }
        
        // Then resolve from registry
        match code {
            "XLM" => Some(AssetRegistry::xlm()),
            "USDC" => Some(AssetRegistry::usdc()),
            "NGNT" => Some(AssetRegistry::ngnt()),
            "USDT" => Some(AssetRegistry::usdt()),
            "EURT" => Some(AssetRegistry::eurt()),
            _ => None,
        }
    }

    /// Check if an asset code is supported
    pub fn is_supported(env: &Env, code: &str) -> bool {
        AssetConfig::is_asset_supported_optimized(env, code)
    }

    /// Get all supported asset codes
    pub fn supported_codes(env: &Env) -> Vec<String> {
        AssetConfig::get_supported_assets(env)
    }

    /// Count supported assets
    pub fn count(env: &Env) -> usize {
        AssetConfig::get_supported_assets_symbols(env).len()
    }

    /// Check if an asset matches by code and issuer
    pub fn matches(env: &Env, code: &str, issuer: &str, asset: &StellarAsset) -> bool {
        // Try to resolve the asset by code
        if let Some(resolved) = Self::resolve_by_code(env, code) {
            // For native XLM, issuer should be empty
            if code == "XLM" {
                return issuer.is_empty() && asset.is_xlm();
            }

            // For non-native assets, check code and issuer match
            asset.code.eq(&resolved.code) && asset.issuer.eq(&resolved.issuer)
        } else {
            false
        }
    }

    /// Get asset metadata along with the asset
    pub fn resolve_with_metadata(
        env: &Env,
        code: &str,
    ) -> Option<(StellarAsset, super::metadata::AssetMetadata)> {
        let asset = Self::resolve_by_code(env, code)?;
        let metadata = MetadataRegistry::get_by_code(code)?;
        Some((asset, metadata))
    }

    /// Validate that an asset is one of our supported assets
    pub fn validate(env: &Env, asset: &StellarAsset) -> bool {
        let code_str = if asset.code.len() == 3 {
            "XLM"
        } else if asset.code.len() == 4 {
            match asset.code.as_raw().as_slice() {
                b"USDC" => "USDC",
                b"NGNT" => "NGNT",
                b"USDT" => "USDT",
                b"EURT" => "EURT",
                _ => return false,
            }
        } else {
            return false;
        };

        if let Some(resolved) = Self::resolve_by_code(env, code_str) {
            asset.code.eq(&resolved.code)
                && asset.issuer.eq(&resolved.issuer)
                && asset.decimals == resolved.decimals
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_by_code() {
        let xlm = AssetResolver::resolve_by_code("XLM");
        assert!(xlm.is_some());
        assert!(xlm.unwrap().is_xlm());

        let usdc = AssetResolver::resolve_by_code("USDC");
        assert!(usdc.is_some());

        let invalid = AssetResolver::resolve_by_code("INVALID");
        assert!(invalid.is_none());
    }

    #[test]
    fn test_is_supported() {
        assert!(AssetResolver::is_supported("XLM"));
        assert!(AssetResolver::is_supported("USDC"));
        assert!(AssetResolver::is_supported("NGNT"));
        assert!(AssetResolver::is_supported("USDT"));
        assert!(AssetResolver::is_supported("EURT"));
        assert!(!AssetResolver::is_supported("INVALID"));
    }

    #[test]
    fn test_supported_codes() {
        let codes = AssetResolver::supported_codes();
        assert_eq!(codes.len(), 5);
    }

    #[test]
    fn test_count() {
        assert_eq!(AssetResolver::count(), 5);
    }
}
