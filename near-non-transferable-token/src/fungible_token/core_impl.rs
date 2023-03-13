
use std::collections::{HashSet, HashMap};
use std::hash::Hash;

use crate::fungible_token::events::{FtMint, FtBurn, FtDeposit, FtWithdraw};
use crate::fungible_token::receiver::ext_ft_receiver;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet, Vector};
use near_sdk::json_types::U128;
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::serde_json::json;
use near_sdk::{
    assert_one_yocto, env, log, require, AccountId, Balance, Gas, IntoStorageKey, PromiseOrValue,
    PromiseResult, StorageUsage, ext_contract, FunctionError,
};

use crate::fungible_token::core::FungibleTokenCore;
use crate::fungible_token::resolver::{FungibleTokenResolver, ext_ft_resolver};
use crate::fungible_token::account::FungibleTokenAccount;

use super::sender::FungibleTokenDeposit;


const GAS_FOR_RESOLVE_BURN: Gas = Gas(5_000_000_000_000);
const GAS_FOR_FT_BURN_CALL: Gas = Gas(25_000_000_000_000 + GAS_FOR_RESOLVE_BURN.0);
const GAS_FOR_RESOLVE_DEPOSIT: Gas = Gas(5_000_000_000_000);
const GAS_FOR_FT_DEPOSIT_CALL: Gas = Gas(25_000_000_000_000 + GAS_FOR_RESOLVE_DEPOSIT.0);
const GAS_FOR_RESOLVE_WITHDRAW: Gas = Gas(5_000_000_000_000);
const GAS_FOR_FT_WITHDRAW_CALL: Gas = Gas(25_000_000_000_000 + GAS_FOR_RESOLVE_WITHDRAW.0);




#[derive(BorshDeserialize, BorshSerialize)]
pub struct Account {
    pub contract_ids: UnorderedMap<Option<AccountId>, (Balance, Balance)>,    //available,  total
    pub deposit_map: UnorderedMap<AccountId, HashMap<Option<AccountId>, Balance>>  //key: specific community drip
}

impl Account {
    pub fn new(prefix: String) -> Self {
        let mut this = Self {
            contract_ids: UnorderedMap::new(prefix.as_bytes()),
            deposit_map: UnorderedMap::new((prefix + "deposit").as_bytes())
        };
        this.contract_ids.insert(&(None as Option<AccountId>), &(0, 0));
        this
    }
}

impl FungibleTokenAccount for Account {

    fn deposit(&mut self, contract_id: &AccountId, amount: Balance) {
        for contract_id  in [Some(contract_id.clone()), None] {
            let balance = self.contract_ids.get(&contract_id).unwrap_or((0, 0));
            if let Some(new_available_balance) = balance.0.checked_add(amount) {
                if let Some(new_total_balance) = balance.1.checked_add(amount) {
                    self.contract_ids.insert(&contract_id, &(new_available_balance, new_total_balance));
                }
            } else {
                env::panic_str("Balance overflow");
            }
        }
    }

    fn withdraw(&mut self, contract_id: &AccountId, amount: Balance) -> u128 {
        for contract_id  in [Some(contract_id.clone()), None] {
            let balance = self.contract_ids.get(&contract_id).expect("not enough balance");
            if let Some(new_available_balance) = balance.0.checked_sub(amount) {
                self.contract_ids.insert(&contract_id, &(new_available_balance, balance.1));
            } else {
                env::panic_str("Not enough balance");
            }
            
        }
        amount
    }

    fn contract_deposit(&mut self, contract_id: &AccountId, deposit_contract_id: &AccountId, amount: Balance) {
        let mut contract = self.deposit_map.get(contract_id).unwrap_or(HashMap::new());
        self.withdraw(contract_id, amount);
        for contract_id  in [Some(deposit_contract_id.clone()), None] {
            let balance = contract.get(&contract_id).unwrap_or(&0).clone();
            if let Some(new_balance) = balance.checked_add(amount) {
                contract.insert(contract_id, new_balance);
            } else {
                env::panic_str("Balance overflow");
            }
        }
        self.deposit_map.insert(contract_id, &contract);
    }

    fn contract_withdraw(&mut self, contract_id: &AccountId, deposit_contract_id: &AccountId, amount: Balance) {
        let mut contract = self.deposit_map.get(contract_id).unwrap_or(HashMap::new());
        for contract_id  in [Some(contract_id.clone()), None] {
            let balance = contract.get(&Some(deposit_contract_id.clone())).expect("not enough balance").clone();
            if let Some(new_balance) = balance.checked_add(amount) {
                contract.insert(contract_id, new_balance);
            } else {
                env::panic_str("Not enough balance");
            }
            
        }
        self.deposit_map.insert(contract_id, &contract);
        self.deposit(contract_id, amount);
    }

    fn get_available_balance(&self, contract_id: &Option<AccountId>) -> u128 {
        match self.contract_ids.get(contract_id) {
            Some(balance) => balance.0,
            None => 0
        }
    }

    fn get_deposit_balance(&self, contract_id: &Option<AccountId>, deposit_contract_id: &Option<AccountId>) -> u128 {
        match contract_id {
            Some(contract_id) => {
                let contract = match self.deposit_map.get(contract_id) {
                    Some(contract) => contract,
                    None => return 0
                };
                match contract.get(deposit_contract_id) {
                    Some(balance) => balance.clone(),
                    None => 0
                }
            }, 
            None => {
                let mut total = 0;
                for (_, deposit) in self.deposit_map.iter() {
                    match deposit.get(&None) {
                        Some(balance) => {
                            total += balance
                        },
                        None => {}
                    };
                }
                total
            },
        }
        
    }

    fn get_total_balance(&self, contract_id: &Option<AccountId>) -> u128 {
        match self.contract_ids.get(contract_id) {
            Some(balance) => balance.1,
            None => 0
        }
    }

    fn is_registered(&self, contract_id: &AccountId) -> bool {
        self.contract_ids.get(&Some(contract_id.clone())).is_some()
    }

    fn is_deposit_exist(&self, cotnract_id: &AccountId, deposit_contract_id: &AccountId) -> bool {
        match self.deposit_map.get(cotnract_id) {
            Some(contract) => {
                contract.get(&Some(deposit_contract_id.clone())).is_some()
            },
            None => false
        }
    }
}

pub type TotalSupply = Account;

/// Implementation of a FungibleToken standard.
/// Allows to include NEP-141 compatible token to any contract.
/// There are next traits that any contract may implement:
///     - FungibleTokenCore -- interface with ft_transfer methods. FungibleToken provides methods for it.
///     - FungibleTokenMetaData -- return metadata for the token in NEP-148, up to contract to implement.
///     - StorageManager -- interface for NEP-145 for allocating storage per account. FungibleToken provides methods for it.
///     - AccountRegistrar -- interface for an account to register and unregister
///
/// For example usage, see examples/fungible-token/src/lib.rs.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct FungibleToken {
    /// AccountID -> Account balance.
    pub accounts: LookupMap<AccountId, Account>,

    /// Total supply of the all token.
    pub total_supply: TotalSupply,

    /// The storage size in bytes for one account.
    pub account_storage_usage: StorageUsage,
}

impl FungibleToken {
    pub fn new<S>(prefix: S) -> Self
    where
        S: IntoStorageKey,
    {
        let mut this =
            Self { accounts: LookupMap::new(prefix), total_supply: TotalSupply::new("total_supply".to_string()), account_storage_usage: 0 };
        this.measure_account_storage_usage();
        this
    }

    fn measure_account_storage_usage(&mut self) {
        let initial_storage_usage = env::storage_usage();
        let tmp_account_id = AccountId::new_unchecked("a".repeat(64));
        self.accounts.insert(&tmp_account_id, &Account::new(tmp_account_id.to_string()));
        self.account_storage_usage = env::storage_usage() - initial_storage_usage;
        env::storage_remove("contracts".as_bytes());
        self.accounts.remove(&tmp_account_id);
    }

    pub fn internal_deposit(&mut self, account_id: &AccountId, amount: Balance, contract_id: &AccountId) {
        let mut account = self.accounts.get(&account_id).expect(format!("The account {} is not registered", &account_id.to_string()).as_str());
        account.deposit(contract_id, amount);
        self.accounts.insert(account_id, &account);
        self.total_supply.deposit(contract_id, amount);

        FtMint {
            owner_id: account_id,
            amount: &amount.into(),
            memo: Some(&json!({
                "contract_id": contract_id
            }).to_string()),
        }
        .emit();
    }

    pub fn internal_withdraw(&mut self, account_id: &AccountId, amount: Balance, contract_id: &AccountId) {
        let mut account = self.accounts.get(&account_id).expect(format!("The account {} is not registered", &account_id.to_string()).as_str());
        let balance = account.get_available_balance(&Some(contract_id.clone()));
        assert!(balance >= amount, "not enough balance");
        account.withdraw(contract_id, amount);
        self.total_supply.withdraw(contract_id, amount);
        self.accounts.insert(account_id, &account);

        FtBurn {
            owner_id: account_id,
            amount: &amount.into(),
            memo: Some(&json!({
                "contract_id": contract_id
            }).to_string()),
        }
        .emit();
    }

    pub fn internal_contract_deposit(&mut self, account_id: &AccountId, amount: Balance, contract_id: &AccountId, deposit_contract_id: &AccountId) {
        let mut account = self.accounts.get(&account_id).expect(format!("The account {} is not registered", &account_id.to_string()).as_str());
        let balance = account.get_available_balance(&Some(contract_id.clone()));
        assert!(balance >= amount, "not enough balance");
        account.contract_deposit(contract_id, deposit_contract_id, amount);

        FtDeposit {
            owner_id: account_id,
            amount: &amount.into(),
            memo: Some(&json!({
                "contract_id": contract_id
            }).to_string()),
        }
        .emit();
    }

    pub fn internal_contract_withdraw(&mut self, account_id: &AccountId, amount: Balance, contract_id: &AccountId, deposit_contract_id: &AccountId) {
        let mut account = self.accounts.get(&account_id).expect(format!("The account {} is not registered", &account_id.to_string()).as_str());
        let deposit_balance = account.get_deposit_balance(&Some(contract_id.clone()), &Some(deposit_contract_id.clone()));
        assert!(deposit_balance >= amount, "not enough balance");
        account.contract_withdraw(contract_id, deposit_contract_id, amount);
        self.accounts.insert(account_id, &account);

        FtWithdraw {
            owner_id: account_id,
            amount: &amount.into(),
            memo: Some(&json!({
                "contract_id": contract_id
            }).to_string()),
        }
        .emit();
    }


    pub fn internal_register_account(&mut self, account_id: &AccountId) {
        if self.accounts.insert(account_id, &Account::new(account_id.to_string())).is_some() {
            env::panic_str("The account is already registered");
        }
    }
    
}

impl FungibleTokenCore for FungibleToken {

    fn ft_available_supply(&self, contract_id: Option<AccountId>) -> U128 {
        self.total_supply.get_total_balance(&contract_id).into()
    }

    fn ft_total_supply(&self, contract_id: Option<AccountId>) -> U128 {
        self.total_supply.get_total_balance(&contract_id).into()
    }

    fn ft_balance_of(&self, account_id: AccountId, contract_id: Option<AccountId>) -> U128 {
        match self.accounts.get(&account_id) {
            Some(account) => account.get_available_balance(&contract_id).into(),
            None => 0.into()
        }
    }

    fn ft_total_balance_of(&self, account_id: AccountId, contract_id: Option<AccountId>) -> U128 {
        match self.accounts.get(&account_id) {
            Some(account) => account.get_total_balance(&contract_id).into(),
            None => 0.into()
        }
    }
}

impl FungibleTokenDeposit for FungibleToken {

    fn ft_deposit_call(
        &mut self,
        receiver_id: AccountId,
        contract_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert!(env::attached_deposit() >= 1, "not enough deposit");
        require!(env::prepaid_gas() > GAS_FOR_FT_DEPOSIT_CALL, "More gas is required");

        let sender_id = env::predecessor_account_id();

        let account = self.accounts.get(&sender_id).expect(format!("The account {} is not registered", &sender_id.to_string()).as_str());
        assert!(account.get_available_balance(&Some(contract_id.clone())) >= amount.0, "not enough balance");

        self.internal_contract_deposit(&sender_id, amount.0, &contract_id, &receiver_id);

        ext_ft_receiver::ext(receiver_id.clone())
        .with_static_gas(env::prepaid_gas() - GAS_FOR_FT_DEPOSIT_CALL)
        .with_attached_deposit(env::attached_deposit())
        .ft_on_deposit(sender_id.clone(), contract_id.clone(), amount.into(), msg)
        .then(
            ext_ft_resolver::ext(env::current_account_id())
                .with_static_gas(GAS_FOR_RESOLVE_BURN)
                .ft_resolve_deposit(sender_id, receiver_id, contract_id, amount),
        )
        .into()
    }

    fn ft_withdraw_call(
        &mut self,
        receiver_id: AccountId,
        contract_id: AccountId,
        amount: U128,
        msg: String
    ) -> PromiseOrValue<U128> {
        assert_one_yocto();
        require!(env::prepaid_gas() > GAS_FOR_FT_WITHDRAW_CALL, "More gas is required");

        let sender_id = env::predecessor_account_id();
        let account = self.accounts.get(&sender_id).expect(format!("The account {} is not registered", &sender_id.to_string()).as_str());
        assert!(account.get_deposit_balance(&Some(contract_id.clone()), &Some(receiver_id.clone())) >= amount.0, "not enough balance");

        ext_ft_receiver::ext(receiver_id.clone())
        .with_static_gas(env::prepaid_gas() - GAS_FOR_FT_WITHDRAW_CALL)
        .with_attached_deposit(env::attached_deposit())
        .ft_on_withdraw(sender_id.clone(), contract_id.clone(), amount.into(), msg)
        .then(
            ext_ft_resolver::ext(env::current_account_id())
                .with_static_gas(GAS_FOR_RESOLVE_BURN)
                .ft_resolve_withdraw(sender_id, receiver_id, contract_id, amount),
        )
        .into()
    }

    fn ft_burn_call(
        &mut self,
        receiver_id: AccountId,
        contract_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert!(env::attached_deposit() >= 1, "not enough deposit");
        require!(env::prepaid_gas() > GAS_FOR_FT_BURN_CALL, "More gas is required");
        let sender_id = env::predecessor_account_id();
        let account = self.accounts.get(&sender_id).expect(format!("The account {} is not registered", &sender_id.to_string()).as_str());
        assert!(account.get_total_balance(&Some(contract_id.clone())) >= amount.0, "not enough balance");

        ext_ft_receiver::ext(receiver_id.clone())
        .with_static_gas(env::prepaid_gas() - GAS_FOR_FT_BURN_CALL)
        .with_attached_deposit(env::attached_deposit())
        .ft_on_burn(sender_id.clone(), contract_id.clone(), amount.into(), msg)
        .then(
            ext_ft_resolver::ext(env::current_account_id())
                .with_static_gas(GAS_FOR_RESOLVE_BURN)
                .ft_resolve_burn(sender_id, contract_id, amount),
        )
        .into()
    }
}

impl FungibleToken {

    fn get_used_amount(&mut self, amount: u128) -> u128 {
        let used_amount: Balance = match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(value) => {
                if let Ok(unused_amount) = near_sdk::serde_json::from_slice::<U128>(&value) {
                    if let Some(used_amount) = amount.checked_sub(unused_amount.0) {
                        used_amount
                    } else {
                        amount
                    }
                } else {
                    0
                }
            }
            PromiseResult::Failed => 0,
        };
        used_amount
    }

    pub fn internal_ft_resolve_deposit(
        &mut self,
        owner_id: &AccountId,
        receiver_id: &AccountId,
        contract_id: &AccountId,
        amount: u128,
    ) -> (u128, u128) {
        let used_amount: Balance = self.get_used_amount(amount);
        if used_amount > 0 {
            self.internal_contract_deposit(owner_id, used_amount, contract_id, receiver_id);
            return (amount, used_amount)
        }
        (amount, 0)
    }

    pub fn internal_ft_resolve_withdraw(
        &mut self,
        owner_id: &AccountId,
        receiver_id: &AccountId,
        contract_id: &AccountId,
        amount: u128,
    ) -> (u128, u128) {
        let used_amount: Balance = self.get_used_amount(amount);
        if used_amount > 0 {
            self.internal_contract_withdraw(owner_id, used_amount, contract_id, receiver_id);
            return (amount, used_amount)
        }
        (amount, 0)
    }

    /// Internal method that returns the amount of burned tokens in a corner case when the sender
    /// has deleted (unregistered) their account while the `ft_transfer_call` was still in flight.
    /// Returns (Used token amount, Burned token amount)
    pub fn internal_ft_resolve_burn(
        &mut self,
        owner_id: &AccountId,
        contract_id: &AccountId,
        amount: u128,
    ) -> (u128, u128) {
        // Get the used amount from the `ft_on_transfer` call result.
        let used_amount: Balance = self.get_used_amount(amount);
        if used_amount > 0 {
            self.internal_withdraw(owner_id, used_amount, &contract_id);
            return (amount, used_amount)
        }
        (amount, 0)
    }
}

impl FungibleTokenResolver for FungibleToken {

    fn ft_resolve_burn(
        &mut self,
        owner_id: AccountId,
        contract_id: AccountId,
        amount: U128,
    ) -> U128 {
        self.internal_ft_resolve_burn(&owner_id, &contract_id, amount.0).0.into()
    }

    fn ft_resolve_deposit(
        &mut self,
        owner_id:AccountId,
        receiver_id: AccountId,
        contract_id:AccountId,
        amount:U128,
    ) -> U128 {
        self.internal_ft_resolve_deposit(&owner_id, &receiver_id, &contract_id, amount.0).0.into()
    }

    fn ft_resolve_withdraw(
        &mut self,
        owner_id:AccountId,
        receiver_id: AccountId,
        contract_id:AccountId,
        amount:U128,
    ) -> U128 {
        self.internal_ft_resolve_withdraw(&owner_id, &receiver_id, &contract_id, amount.0).0.into()
    }

}
