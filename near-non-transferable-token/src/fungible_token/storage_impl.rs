use crate::storage_management::{StorageBalance, StorageBalanceBounds, StorageManagement};
use near_sdk::json_types::U128;
use near_sdk::{assert_one_yocto, env, log, AccountId, Balance, Promise};

use crate::fungible_token::core_impl::FungibleToken;
use crate::fungible_token::account::FungibleTokenAccount;

impl FungibleToken {
    /// Internal method that returns the Account ID and the balance in case the account was
    /// unregistered.
    pub fn internal_storage_unregister(
        &mut self,
        force: Option<bool>,
    ) -> Option<(AccountId, Balance)> {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        let force = force.unwrap_or(false);
        let mut account = self.accounts.get(&account_id).expect(format!("The account {} is not registered", &account_id.to_string()).as_str());
        let balance = account.get_available_balance(&None);
        let deposit_balance = account.get_deposit_balance(&None, &None);
        if balance > 0 {
            if force {
                for (contract_id, amount) in account.contract_ids.iter() {
                    if let Some(contract_id) = contract_id {
                        self.total_supply.withdraw(&contract_id, amount); 
                    }
                }
            } else {
                env::panic_str(
                    "Can't unregister the account with the positive balance without force",
                )
            }
        }
        
        if deposit_balance > 0 {
            env::panic_str(
                "Can't unregister the account with the positive balance without force",
            )
        }
        account.contract_ids.clear();
        account.deposit_map.clear();
        self.accounts.remove(&account_id);
        Promise::new(account_id.clone()).transfer(self.storage_balance_bounds().min.0 + 1);
        Some((account_id, balance))
    }

    fn internal_storage_balance_of(&self, account_id: &AccountId, include_deposit_contracts: bool) -> Option<StorageBalance> {
        if self.accounts.contains_key(account_id) {
            let account = self.accounts.get(&account_id).unwrap();
            let mut contract_count = account.contract_ids.len() - 1;
            if include_deposit_contracts {
                contract_count += account.deposit_map.len();
            }
            let total = contract_count as u128 * self.storage_balance_bounds().min.0;
            Some(StorageBalance { total: total.into(), available: 0.into() })
        } else {
            None
        }
    }
}

impl StorageManagement for FungibleToken {
    // `registration_only` doesn't affect the implementation for vanilla fungible token.
    #[allow(unused_variables)]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        let amount: Balance = env::attached_deposit();
        let account_id = account_id.unwrap_or_else(env::predecessor_account_id);
        if self.accounts.contains_key(&account_id) {
            log!("The account is already registered, refunding the deposit");
            if amount > 0 {
                Promise::new(env::predecessor_account_id()).transfer(amount);
            }
        } else {
            let min_balance = self.storage_balance_bounds().min.0;
            if amount < min_balance {
                env::panic_str("The attached deposit is less than the minimum storage balance");
            }

            self.internal_register_account(&account_id);
            let refund = amount - min_balance;
            if refund > 0 {
                Promise::new(env::predecessor_account_id()).transfer(refund);
            }
        }
        self.internal_storage_balance_of(&account_id, false).unwrap()
    }

    /// While storage_withdraw normally allows the caller to retrieve `available` balance, the basic
    /// Fungible Token implementation sets storage_balance_bounds.min == storage_balance_bounds.max,
    /// which means available balance will always be 0. So this implementation:
    /// * panics if `amount > 0`
    /// * never transfers Ⓝ to caller
    /// * returns a `storage_balance` struct if `amount` is 0
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        assert_one_yocto();
        let predecessor_account_id = env::predecessor_account_id();
        if let Some(storage_balance) = self.internal_storage_balance_of(&predecessor_account_id, true) {
            match amount {
                Some(amount) if amount.0 > 0 => {
                    env::panic_str("The amount is greater than the available storage balance");
                }
                _ => storage_balance,
            }
        } else {
            env::panic_str(
                format!("The account {} is not registered", &predecessor_account_id).as_str(),
            );
        }
    }

    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        self.internal_storage_unregister(force).is_some()
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        let required_storage_balance =
            Balance::from(self.account_storage_usage) * env::storage_byte_cost();
        StorageBalanceBounds {
            min: required_storage_balance.into(),
            max: Some(required_storage_balance.into()),
        }
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        self.internal_storage_balance_of(&account_id, true)
    }
}
