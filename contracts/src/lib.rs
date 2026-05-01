#![no_std]

/// EventMesh Soroban smart contracts
/// Composable on-chain event infrastructure for ticketing, payments, and vendor coordination

/// EventMeshNFT: Soulbound NFT for event attendance and ticketing
pub mod nft;

// Re-export public types for upstream contracts
pub use nft::{EventMeshNFT, NFTMetadata, DataKey};
pub mod factory;
pub use factory::Factory;
