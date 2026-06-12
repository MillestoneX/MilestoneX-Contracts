//! Common types shared across the OrbitChain workspace.
//!
//! This crate provides canonical definitions for CampaignStatus, MilestoneStatus,
//! AssetInfo, and ErrorCode used by both campaign and core contracts.

#![no_std]
use soroban_sdk::{contracttype, contracterror};

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CampaignStatus {
    Draft,
    Active,
    Completed,
    Cancelled,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MilestoneStatus {
    Pending,
    Completed,
    Failed,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct AssetInfo {
    pub code: u32,
    pub issuer: u32,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
}
