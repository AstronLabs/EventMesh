use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventInfo {
    pub nft_contract: Address,
    pub owner: Address,
    pub name: String,
    pub location: String,
    pub details: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    EventCount,
    EventAt(u32),
}

#[contract]
pub struct Factory;

#[contractimpl]
impl Factory {
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::EventCount, &0u32);
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Factory admin not initialized"))
    }

    /// Create a new Event entry and deploy a fresh EventMeshNFT instance for it.
    /// Returns the created index (u32) and the NFT contract address.
    pub fn create_event(
        env: Env,
        owner: Address,
        name: String,
        location: String,
        details: String,
    ) -> (u32, Address) {
        // Only factory admin can create events
        let admin = Self::get_admin(env.clone());
        admin.require_auth();

        // Increment count and compute index
        let mut count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::EventCount)
            .unwrap_or(0);
        let index = count;

        // Deploy a fresh NFT contract instance for the event
        // We register a new EventMeshNFT instance (defined in nft.rs)
        let nft_contract = env.register(crate::nft::EventMeshNFT, ());

        // Store event info
        let info = EventInfo {
            nft_contract: nft_contract.clone(),
            owner: owner.clone(),
            name: name.clone(),
            location: location.clone(),
            details: details.clone(),
        };
        env.storage()
            .instance()
            .set(&DataKey::EventAt(index), &info);

        // update count
        count = count.saturating_add(1);
        env.storage()
            .instance()
            .set(&DataKey::EventCount, &count);

        (index, nft_contract)
    }

    pub fn event_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::EventCount)
            .unwrap_or(0)
    }

    pub fn get_event(env: Env, index: u32) -> EventInfo {
        env.storage()
            .instance()
            .get(&DataKey::EventAt(index))
            .unwrap_or_else(|| panic!("Event not found"))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_factory_create_event() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let owner = Address::generate(&env);

        let contract_id = env.register(Factory, ());
        let client = FactoryClient::new(&env, &contract_id);

        client.initialize(&admin);
        assert_eq!(client.get_admin(), admin);

        let name = String::from_str(&env, "Conf");
        let location = String::from_str(&env, "City");
        let details = String::from_str(&env, "Details");

        let (idx, nft_addr) = client.create_event(&owner, &name, &location, &details);
        assert_eq!(idx, 0);
        assert_eq!(client.event_count(), 1);

        let info = client.get_event(&idx);
        assert_eq!(info.owner, owner);
        assert_eq!(info.name, name);
        assert_eq!(info.location, location);
        assert_eq!(info.details, details);
        assert_eq!(info.nft_contract, nft_addr);
    }
}
