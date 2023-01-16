<div align="center">

  <h1><code>near-non-transferable-token</code></h1>

  <p>
    <strong>Popula Library for Non-transferable Token.</strong>
  </p>


  <p>
    <a href="https://crates.io/crates/near-non-transferable-token"><img src="https://img.shields.io/crates/v/near-non-transferable-token.svg?style=flat-square" alt="Crates.io version" /></a>
    <a href="https://crates.io/crates/near-non-transferable-token"><img src="https://img.shields.io/crates/d/near-non-transferable-token.svg?style=flat-square" alt="Download" /></a>
    <a href="https://docs.rs/near-non-transferable-token"><img src="https://docs.rs/near-non-transferable-token/badge.svg" alt="Reference Documentation" /></a>
  </p>


</div>

## Example


```rust
use near_non_transferable_token::{impl_fungible_token_core, impl_fungible_token_storage};

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        owner_id: AccountId,
        metadata: FungibleTokenMetadata,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
            owner_id,
            white_list: HashSet::new()
        };
        this
    }
}

impl_fungible_token_core!(Contract, token);
impl_fungible_token_storage!(Contract, token);


#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}
```
See https://github.com/beepopula/Drip-contract for more details.

## Features

### Account Book
Separate Balances for different contracts:

```rust
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Account {
    pub contract_ids: UnorderedMap<Option<AccountId>, Balance>,
    pub deposit_map: UnorderedMap<AccountId, HashMap<Option<AccountId>, Balance>>  //key: specific community drip
}
```

NOTES:
 - If the key for contract_ids is None then it represent the sum of Balance.
 - Deposit is a derivative function for proving that you have such balance for only once, preventing infinite proving. And the ownership should remain the same. 

## Versioning

### Semantic Versioning

This crate follows [Cargo's semver guidelines](https://doc.rust-lang.org/cargo/reference/semver.html). 

State breaking changes (low-level serialization format of any data type) will be avoided at all costs. If a change like this were to happen, it would come with a major version and come with a compiler error. If you encounter one that does not, [open an issue](https://github.com/near/near-non-transferable-token-rs/issues/new)!

### MSRV

The minimum supported Rust version is currently `1.56`. There are no guarantees that this will be upheld if a security patch release needs to come in that requires a Rust toolchain increase.
