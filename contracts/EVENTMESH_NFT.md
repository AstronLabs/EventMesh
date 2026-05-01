# EventMeshNFT Contract

## Overview

EventMeshNFT is a Soroban smart contract that implements soulbound (non-transferable) NFTs for event ticketing and attendance verification on the Stellar network.

**Key Property:** Each address can hold exactly one NFT per contract instance. Once minted, an NFT is permanently bound to its holder's address and cannot be transferred.

## Contract Architecture

### Data Structures

#### NFTMetadata

Immutable metadata attached to each minted NFT:

```rust
pub struct NFTMetadata {
    pub event_name: String,      // Event name (e.g., "TechConf 2026")
    pub location: String,         // City name (e.g., "San Francisco")
    pub owner: Address,           // Event organizer/owner address
    pub event_details: String,    // Event description/details
}
```

#### DataKey Enum

Storage keys for contract state:

- `Admin` - The authorized administrator address (can mint/burn)
- `NFTHolder(Address)` - Maps to NFTMetadata for a specific holder
- `TotalSupply` - Counter of total NFTs minted

### Error Codes

| Code | Constant               | Meaning                                            |
| ---- | ---------------------- | -------------------------------------------------- |
| 1    | `UNAUTHORIZED`         | Caller lacks required authorization                |
| 2    | `ALREADY_MINTED`       | Address already holds an NFT (soulbound violation) |
| 3    | `NOT_MINTED`           | Address does not hold an NFT                       |
| 4    | `TRANSFER_NOT_ALLOWED` | NFTs cannot be transferred (soulbound)             |
| 5    | `INVALID_METADATA`     | One or more metadata fields are empty              |

## Public Functions

### `initialize(env, admin) -> void`

Initialize the contract with an administrator address.

**Parameters:**

- `admin: Address` - The address granted minting/burning privileges

**Authorization Required:** Admin must authorize this call

**Effects:**

- Sets the contract admin
- Initializes supply counter to 0

**Panics:** If admin not already initialized (only call once per contract)

---

### `mint(env, holder, event_name, location, owner, event_details) -> u32`

Mint a new soulbound NFT to a holder address.

**Parameters:**

- `holder: Address` - The recipient of the NFT
- `event_name: String` - Name of the event (cannot be empty)
- `location: String` - City name (cannot be empty)
- `owner: Address` - Event organizer address
- `event_details: String` - Event description (cannot be empty)

**Authorization Required:** Admin must authorize this call

**Returns:**

- `0` - Success
- `ALREADY_MINTED` - Holder already has an NFT
- `INVALID_METADATA` - One or more string fields are empty

**Effects (on success):**

- Stores NFTMetadata for the holder
- Increments total supply counter
- Prevents future mints to the same holder

**Soulbound Guarantee:** Each address can only hold one NFT per contract instance.

---

### `get_nft(env, holder) -> NFTMetadata`

Retrieve the NFT metadata for a holder address.

**Parameters:**

- `holder: Address` - The NFT holder address

**Returns:** NFTMetadata struct with event information

**Panics:** If the holder doesn't have an NFT

---

### `holds_nft(env, holder) -> bool`

Check if an address currently holds an NFT.

**Parameters:**

- `holder: Address` - Address to check

**Returns:**

- `true` - Address holds an NFT
- `false` - Address does not hold an NFT

---

### `total_supply(env) -> u32`

Get the total number of NFTs minted on this contract instance.

**Returns:** Current supply count

---

### `burn(env, holder) -> u32`

Revoke/burn an NFT from a holder (admin-only operation).

**Parameters:**

- `holder: Address` - Address whose NFT will be revoked

**Authorization Required:** Admin must authorize this call

**Returns:**

- `0` - Success
- `NOT_MINTED` - Holder does not have an NFT

**Effects (on success):**

- Removes NFT metadata for the holder
- Decrements total supply counter

**Use Cases:**

- Invalidate fraudulent attendance
- Administrative correction of minting errors

---

### `transfer(env, from, to) -> u32`

Attempt to transfer an NFT (always fails).

**Returns:** Always returns `TRANSFER_NOT_ALLOWED` (4)

**Purpose:** Explicit guard to prevent accidental transfer attempts. Soulbound NFTs cannot be transferred under any circumstances.

---

## Design Rationale

### Soulbound Implementation

NFTs are inherently soulbound through contract design:

1. **One per address:** Storage key includes the holder address, preventing multiple NFTs per holder
2. **No transfer function:** Any transfer attempt returns an error
3. **Immutable metadata:** Once minted, metadata cannot be changed (only burn removes it)

### Storage Pattern

- Uses `instance()` storage (not persistent, cleared with contract reset)
- O(1) lookup for NFT existence checks
- Efficient supply tracking with a counter

### Access Control

- Only admin can mint and burn
- All minting requires explicit `require_auth()`
- No public minting or burning

### Error Handling

Error codes are returned as u32 (Soroban contract pattern):

- `0` indicates success
- Non-zero values indicate specific errors
- Callers must check return values to determine operation outcome

## Integration Example

### With an Event Factory Contract

The Event Factory would:

1. Create a new EventMeshNFT instance for each event
2. Pass the Event contract as the admin
3. Event contract calls `mint()` when USDT payment is received
4. Event contract calls `burn()` if refunds are issued

```rust
// Pseudocode
let event_nft = instantiate_eventmesh_nft();
event_nft.initialize(&self.address); // Event contract is admin
// ... later, on payment ...
event_nft.mint(&payer, &event_name, &location, &self.owner, &details);
```

## Security Considerations

1. **Admin Key Management:** Ensure the admin address is a secure multi-sig or trusted entity
2. **Metadata Validation:** All string fields must be non-empty (contract enforces this)
3. **Soulbound Guarantee:** The contract design makes the soulbound property cryptographically enforced
4. **Supply Accuracy:** Burning is always balanced with minting to maintain supply integrity

## Test Coverage

6 comprehensive unit tests included:

- ✅ Initialization
- ✅ Minting with valid metadata
- ✅ Double-minting prevention (soulbound guarantee)
- ✅ Burning and supply updates
- ✅ Transfer rejection
- ✅ Invalid metadata rejection

All tests pass: `cargo test`

## Deployment

### Compile for WASM

```bash
cd contracts
cargo build --target wasm32-unknown-unknown --release
```

### Run Tests

```bash
cargo test
```

Output: `target/wasm32-unknown-unknown/release/eventmesh_contracts.wasm`

## Future Enhancements (Not Currently Implemented)

- Batch minting for gas efficiency
- Metadata URI (pointing to external data)
- Event-based logging for off-chain indexing
- Admin transfer capability
