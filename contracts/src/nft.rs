use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String};

/// EventMeshNFT: Soulbound NFT for event ticketing
/// Each NFT represents paid attendance at an event and contains immutable metadata.
/// Soulbound means non-transferable—once issued to an address, it cannot be moved.

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NFTMetadata {
    /// Event name
    pub event_name: String,
    /// Event location (city name)
    pub location: String,
    /// Event owner/organizer address
    pub owner: Address,
    /// Event details (description, date, etc.)
    pub event_details: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Admin address authorized to mint NFTs
    Admin,
    /// Maps holder address to their NFT metadata (one NFT per address per event)
    NFTHolder(Address),
    /// Total NFTs issued count
    TotalSupply,
}

/// Error codes for EventMeshNFT operations
/// Used for contract invocation error handling
pub mod error {
    pub const UNAUTHORIZED: u32 = 1;
    pub const ALREADY_MINTED: u32 = 2;
    pub const NOT_MINTED: u32 = 3;
    pub const TRANSFER_NOT_ALLOWED: u32 = 4;
    pub const INVALID_METADATA: u32 = 5;
}

#[contract]
pub struct EventMeshNFT;

#[contractimpl]
impl EventMeshNFT {
    /// Initialize the contract with an admin address.
    /// The admin is the only address that can mint new NFTs.
    pub fn initialize(env: Env, admin: Address) {
        // Verify caller has auth context
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TotalSupply, &0u32);
    }

    /// Get the current admin address.
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Admin not initialized"))
    }

    /// Mint a new soulbound NFT to a holder address.
    /// Each address can only hold one NFT per event contract instance.
    /// Only the admin can mint.
    /// Returns 0 on success, or error code otherwise.
    pub fn mint(
        env: Env,
        holder: Address,
        event_name: String,
        location: String,
        owner: Address,
        event_details: String,
    ) -> u32 {
        // Verify admin authorization
        let admin = Self::get_admin(env.clone());
        admin.require_auth();

        // Validate inputs
        if event_name.is_empty() || location.is_empty() || event_details.is_empty() {
            return error::INVALID_METADATA;
        }

        // Check if holder already has an NFT (soulbound - only one per address)
        if env
            .storage()
            .instance()
            .has(&DataKey::NFTHolder(holder.clone()))
        {
            return error::ALREADY_MINTED;
        }

        // Create and store NFT metadata
        let metadata = NFTMetadata {
            event_name,
            location,
            owner,
            event_details,
        };

        env.storage()
            .instance()
            .set(&DataKey::NFTHolder(holder), &metadata);

        // Increment supply counter
        let supply: u32 = env
            .storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &(supply + 1));

        0 // Success
    }

    /// Get NFT metadata for a holder address.
    /// Returns the metadata if the holder has an NFT, panics if not.
    pub fn get_nft(env: Env, holder: Address) -> NFTMetadata {
        env.storage()
            .instance()
            .get(&DataKey::NFTHolder(holder))
            .unwrap_or_else(|| panic!("NFT not found for holder"))
    }

    /// Check if an address holds an NFT.
    pub fn holds_nft(env: Env, holder: Address) -> bool {
        env.storage()
            .instance()
            .has(&DataKey::NFTHolder(holder))
    }

    /// Get total number of NFTs issued.
    pub fn total_supply(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0)
    }

    /// Burn (revoke) an NFT. Only admin can do this, e.g., for invalid attendance.
    /// Returns 0 on success, or error code otherwise.
    pub fn burn(env: Env, holder: Address) -> u32 {
        let admin = Self::get_admin(env.clone());
        admin.require_auth();

        if !env
            .storage()
            .instance()
            .has(&DataKey::NFTHolder(holder.clone()))
        {
            return error::NOT_MINTED;
        }

        env.storage()
            .instance()
            .remove(&DataKey::NFTHolder(holder));

        let supply: u32 = env
            .storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &supply.saturating_sub(1));

        0 // Success
    }

    /// Attempt a transfer—this will always fail because NFTs are soulbound.
    /// This is an explicit guard to prevent accidental transfer attempts.
    /// Always returns TRANSFER_NOT_ALLOWED error code.
    pub fn transfer(_env: Env, _from: Address, _to: Address) -> u32 {
        error::TRANSFER_NOT_ALLOWED
    }

    /// Allow the current admin to transfer admin role to `new_admin`.
    /// Returns 0 on success or `UNAUTHORIZED` if caller is not current admin.
    pub fn set_admin(env: Env, new_admin: Address) -> u32 {
        let current = Self::get_admin(env.clone());
        // current admin must authorize this change
        current.require_auth();
        env.storage().instance().set(&DataKey::Admin, &new_admin);
        0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_initialize() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register(EventMeshNFT, ());
        let client = EventMeshNFTClient::new(&env, &contract_id);

        client.initialize(&admin);

        let stored_admin = client.get_admin();
        assert_eq!(stored_admin, admin);

        assert_eq!(client.total_supply(), 0);
    }

    #[test]
    fn test_mint_nft() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let holder = Address::generate(&env);
        let owner = Address::generate(&env);

        let contract_id = env.register(EventMeshNFT, ());
        let client = EventMeshNFTClient::new(&env, &contract_id);

        client.initialize(&admin);

        let event_name = String::from_str(&env, "TechConf 2026");
        let location = String::from_str(&env, "San Francisco");
        let event_details = String::from_str(&env, "Annual tech conference");

        let result = client.mint(&holder, &event_name, &location, &owner, &event_details);
        assert_eq!(result, 0); // Success

        assert!(client.holds_nft(&holder));
        assert_eq!(client.total_supply(), 1);

        let metadata = client.get_nft(&holder);
        assert_eq!(metadata.event_name, event_name);
        assert_eq!(metadata.location, location);
        assert_eq!(metadata.owner, owner);
        assert_eq!(metadata.event_details, event_details);
    }

    #[test]
    fn test_prevent_double_mint() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let holder = Address::generate(&env);
        let owner = Address::generate(&env);

        let contract_id = env.register(EventMeshNFT, ());
        let client = EventMeshNFTClient::new(&env, &contract_id);

        client.initialize(&admin);

        let event_name = String::from_str(&env, "Event1");
        let location = String::from_str(&env, "NYC");
        let event_details = String::from_str(&env, "Details");

        // First mint succeeds
        let result1 = client.mint(&holder, &event_name, &location, &owner, &event_details);
        assert_eq!(result1, 0);

        // Second mint to same holder fails (soulbound)
        let result2 = client.mint(&holder, &event_name, &location, &owner, &event_details);
        assert_eq!(result2, error::ALREADY_MINTED);
    }

    #[test]
    fn test_burn_nft() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let holder = Address::generate(&env);
        let owner = Address::generate(&env);

        let contract_id = env.register(EventMeshNFT, ());
        let client = EventMeshNFTClient::new(&env, &contract_id);

        client.initialize(&admin);

        let event_name = String::from_str(&env, "Event1");
        let location = String::from_str(&env, "NYC");
        let event_details = String::from_str(&env, "Details");

        client.mint(&holder, &event_name, &location, &owner, &event_details);
        assert_eq!(client.total_supply(), 1);

        let burn_result = client.burn(&holder);
        assert_eq!(burn_result, 0); // Success

        assert!(!client.holds_nft(&holder));
        assert_eq!(client.total_supply(), 0);
    }

    #[test]
    fn test_soulbound_transfer_blocked() {
        let env = Env::default();
        env.mock_all_auths();

        let from = Address::generate(&env);
        let to = Address::generate(&env);

        let contract_id = env.register(EventMeshNFT, ());
        let client = EventMeshNFTClient::new(&env, &contract_id);

        let result = client.transfer(&from, &to);
        assert_eq!(result, error::TRANSFER_NOT_ALLOWED);
    }

    #[test]
    fn test_invalid_metadata() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let holder = Address::generate(&env);
        let owner = Address::generate(&env);

        let contract_id = env.register(EventMeshNFT, ());
        let client = EventMeshNFTClient::new(&env, &contract_id);

        client.initialize(&admin);

        // Attempt mint with empty event name
        let empty_string = String::from_str(&env, "");
        let valid_string = String::from_str(&env, "Valid");

        let result = client.mint(&holder, &empty_string, &valid_string, &owner, &valid_string);
        assert_eq!(result, error::INVALID_METADATA);
    }
}
