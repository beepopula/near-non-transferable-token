
use std::collections::{HashSet, HashMap};
use std::hash::Hash;

use crate::fungible_token::events::{FtMint, FtBurn, FtDeposit};
use crate::fungible_token::receiver::ext_ft_receiver;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet, Vector};
use near_sdk::json_types::U128;
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::serde_json::json;
use near_sdk::{
    assert_one_yocto, env, log, require, AccountId, Balance, Gas, IntoStorageKey, PromiseOrValue,
    PromiseResult, StorageUsage, ext_contract,
};

use crate::fungible_token::core::{TokenSource, FungibleTokenAccount};
use crate::fungible_token::core::FungibleTokenCore;
use crate::fungible_token::resolver::{FungibleTokenResolver, ext_ft_resolver};


const GAS_FOR_RESOLVE_BURN: Gas = Gas(5_000_000_000_000);
const GAS_FOR_FT_BURN_CALL: Gas = Gas(25_000_000_000_000 + GAS_FOR_RESOLVE_BURN.0);
const GAS_FOR_RESOLVE_DEPOSIT: Gas = Gas(5_000_000_000_000);
const GAS_FOR_FT_DEPOSIT_CALL: Gas = Gas(25_000_000_000_000 + GAS_FOR_RESOLVE_DEPOSIT.0);




#[derive(BorshDeserialize, BorshSerialize)]
pub struct Account {
    pub contract_ids: UnorderedMap<Option<AccountId>, HashMap<TokenSource, Balance>>,
    pub deposit_map: UnorderedMap<AccountId, HashMap<Option<AccountId>, HashMap<TokenSource, Balance>>>  //key: specific community drip
}

impl Account {
    pub fn new(prefix: String) -> Self {
        let mut balance = HashMap::new();
        balance.insert(TokenSource::ApplicationValue, 0 as Balance);
        balance.insert(TokenSource::FinancialValue, 0 as Balance);
        let mut this = Self {
            contract_ids: UnorderedMap::new(prefix.as_bytes()),
            deposit_map: UnorderedMap::new((prefix + "deposit").as_bytes())
        };
        this.contract_ids.insert(&(None as Option<AccountId>), &balance);
        this
    }
}

impl FungibleTokenAccount for Account {

    fn deposit(&mut self, contract_id: &AccountId, token_source: &TokenSource, amount: Balance) {
        for contract_id  in [Some(contract_id.clone()), None] {
            let mut token_map = self.contract_ids.get(&contract_id).unwrap_or(HashMap::new());
            let balance = token_map.get(&token_source).unwrap_or(&(0 as Balance)).clone();
            if let Some(new_balance) = balance.checked_add(amount) {
                token_map.insert(token_source.clone(), new_balance);
            } else {
                env::panic_str("Balance overflow");
            }
            self.contract_ids.insert(&contract_id, &token_map);
        }
    }

    fn withdraw(&mut self, contract_id: &AccountId, token_source: &TokenSource, amount: Balance) -> u128 {
        for contract_id  in [Some(contract_id.clone()), None] {
            let mut token_map = self.contract_ids.get(&contract_id).expect("not enough balance");
            let balance = token_map.get(token_source).unwrap_or(&(0 as Balance)).clone();
            if let Some(new_balance) = balance.checked_sub(amount) {
                token_map.insert(token_source.clone(), new_balance);
            } else {
                env::panic_str("Not enough balance");
            }
            self.contract_ids.insert(&contract_id, &token_map);
        }
        amount
    }

    fn contract_deposit(&mut self, contract_id: &AccountId, deposit_contract_id: &AccountId, token_source: &TokenSource, amount: Balance) {
        let mut contract = self.deposit_map.get(contract_id).unwrap_or(HashMap::new());
        for contract_id  in [Some(deposit_contract_id.clone()), None] {
            let mut token_map = contract.get(&contract_id).unwrap_or(&HashMap::new()).clone();
            let balance = token_map.get(&token_source).unwrap_or(&(0 as Balance)).clone();
            if let Some(new_balance) = balance.checked_add(amount) {
                token_map.insert(token_source.clone(), new_balance);
            } else {
                env::panic_str("Balance overflow");
            }
            contract.insert(contract_id, token_map.clone());
        }
        self.deposit_map.insert(contract_id, &contract);
    }

    fn contract_withdraw(&mut self, contract_id: &AccountId, deposit_contract_id: &Option<AccountId>, token_source: &TokenSource, amount: Balance) -> Vec<(AccountId, TokenSource, u128)> {
        let mut contract = self.deposit_map.get(contract_id).unwrap_or(HashMap::new());
        match deposit_contract_id {
            Some(deposit_contract_id) => {
                for contract_id  in [Some(contract_id.clone()), None] {
                    let mut token_map = contract.get(&Some(deposit_contract_id.clone())).expect("not enough balance").clone();
                    let balance = token_map.get(&token_source).unwrap_or(&(0 as Balance)).clone();
                    if let Some(new_balance) = balance.checked_add(amount) {
                        token_map.insert(token_source.clone(), new_balance);
                    } else {
                        env::panic_str("Not enough balance");
                    }
                    contract.insert(contract_id, token_map.clone());
                }
                self.deposit_map.insert(contract_id, &contract);
                vec![(deposit_contract_id.clone(), token_source.clone(), amount)]
            },
            None => {
                let withdraw_contract_ids = vec![];
                let mut left_amount = amount.clone();
                for deposit_contract_id in contract.clone().keys() {
                    match deposit_contract_id {
                        Some(deposit_contract_id) => {
                            let mut token_map = contract.get(&Some(deposit_contract_id.clone())).expect("not enough balance").clone();
                            let balance = token_map.get(&token_source).unwrap_or(&(0 as Balance)).clone();
                            if let Some(new_balance) = balance.checked_add(left_amount) {
                                token_map.insert(token_source.clone(), new_balance);
                            } else {
                                left_amount -= balance;
                            }
                            contract.insert(Some(deposit_contract_id.clone()), token_map.clone());
                            self.deposit_map.insert(contract_id, &contract);
                        },
                        None => {
                            let mut token_map = contract.get(&None).expect("not enough balance").clone();
                            let balance = token_map.get(&token_source).unwrap_or(&(0 as Balance)).clone();
                            if let Some(new_balance) = balance.checked_add(left_amount) {
                                token_map.insert(token_source.clone(), new_balance);
                            } else {
                                env::panic_str("Not enough balance");
                            }
                        }
                    }
                }
                withdraw_contract_ids
            }
        }
    }

    fn get_balance(&self, contract_id: &Option<AccountId>, token_source: &Option<TokenSource>) -> u128 {
        match self.contract_ids.get(contract_id) {
            Some(token_map) => {
                match token_source {
                    Some(token_source) => *token_map.get(token_source).unwrap_or(&0),
                    None => {
                        let mut total = 0;
                        for (_, balance) in token_map {
                            total += balance;
                        }
                        total
                    }
                }
                
            },
            None => 0
        }
    }

    fn get_deposit_balance(&self, contract_id: &AccountId, token_source: &Option<TokenSource>) -> u128 {
        let contract = self.deposit_map.get(contract_id).unwrap();
        match contract.get(&None) {
            Some(token_map) => {
                match token_source {
                    Some(token_source) => *token_map.get(token_source).unwrap_or(&0),
                    None => {
                        let mut total = 0;
                        for (_, balance) in token_map {
                            total += balance;
                        }
                        total
                    }
                }
                
            },
            None => 0
        }
    }

    fn is_registered(&self, contract_id: &AccountId) -> bool {
        self.contract_ids.get(&Some(contract_id.clone())).is_some()
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

    pub fn internal_deposit(&mut self, account_id: &AccountId, amount: Balance, contract_id: &AccountId, token_source: &TokenSource) {
        let mut account = self.accounts.get(&account_id).expect(format!("The account {} is not registered", &account_id.to_string()).as_str());
        account.deposit(contract_id, token_source, amount);
        self.accounts.insert(account_id, &account);

        self.total_supply.deposit(contract_id, token_source, amount);

        FtMint {
            owner_id: account_id,
            amount: &amount.into(),
            memo: Some(&json!({
                "contract_id": contract_id
            }).to_string()),
        }
        .emit();
    }

    pub fn internal_contract_deposit(&mut self, account_id: &AccountId, amount: Balance, contract_id: &AccountId, deposit_contract_id: &AccountId, token_source: &Option<TokenSource>) {
        let mut account = self.accounts.get(&account_id).expect(format!("The account {} is not registered", &account_id.to_string()).as_str());
        match token_source {
            Some(token_source) => {
                let balance = account.get_balance(&Some(contract_id.clone()), &Some(token_source.clone()));
                assert!(balance >= amount, "not enough balance");
                account.withdraw(contract_id, &TokenSource::FinancialValue, amount);
                account.contract_deposit(contract_id, deposit_contract_id, token_source, amount)
            },
            None => {
                let financial_balance = account.get_balance(&Some(contract_id.clone()), &Some(TokenSource::FinancialValue));
                let application_balance = account.get_balance(&Some(contract_id.clone()), &Some(TokenSource::ApplicationValue));
                assert!(financial_balance + application_balance >= amount, "not enough balance");
                if let Some(new_financial_balance) = financial_balance.checked_sub(amount) {
                    account.withdraw(contract_id, &TokenSource::FinancialValue, amount);
                    account.contract_deposit(contract_id, deposit_contract_id, &TokenSource::FinancialValue, amount);
                } else {
                    account.withdraw(contract_id, &TokenSource::FinancialValue, financial_balance);
                    account.contract_deposit(contract_id, deposit_contract_id, &TokenSource::FinancialValue, amount);
                    if let Some(new_application_balance) = application_balance.checked_sub(amount - financial_balance) {
                        account.withdraw(contract_id, &TokenSource::ApplicationValue, new_application_balance);
                        account.contract_deposit(contract_id, deposit_contract_id, &TokenSource::ApplicationValue, amount);
                    }
                }
            }
            
        }
    }

    pub fn wrapped_withdraw(&mut self, account_id: &AccountId, amount: Balance, contract_id: &AccountId, token_source: &Option<TokenSource>) -> Vec<(AccountId, TokenSource, u128)> {
        let mut account = self.accounts.get(&account_id).expect(format!("The account {} is not registered", &account_id.to_string()).as_str());
        let balance = account.get_balance(&Some(contract_id.clone()), token_source);
        let mut deposit_contract_ids = vec![];
        if let Some(new_balance) = balance.checked_sub(amount) {
            self.withdraw(account_id, amount, contract_id, token_source);
        } else {
            let deposit_balance = account.get_deposit_balance(contract_id, token_source);
            if let Some(new_deposit_balance) = deposit_balance.checked_sub(amount - balance) {
                deposit_contract_ids = self.contract_withdraw(account_id, amount - balance, contract_id, &None, token_source)
            } else {
                panic!("not enough balance");
            }
            
        }
        self.accounts.insert(account_id, &account);
        deposit_contract_ids
    }




    fn withdraw(&mut self, account_id: &AccountId, amount: Balance, contract_id: &AccountId, token_source: &Option<TokenSource>) {
        let mut account = self.accounts.get(&account_id).expect(format!("The account {} is not registered", &account_id.to_string()).as_str());
        match token_source {
            Some(token_source) => {
                let balance = account.get_balance(&Some(contract_id.clone()), &Some(token_source.clone()));
                assert!(balance >= amount, "not enough balance");
                account.withdraw(contract_id, &TokenSource::FinancialValue, amount);
                self.total_supply.withdraw(contract_id, &TokenSource::FinancialValue, amount);
            },
            None => {
                let financial_balance = account.get_balance(&Some(contract_id.clone()), &Some(TokenSource::FinancialValue));
                let application_balance = account.get_balance(&Some(contract_id.clone()), &Some(TokenSource::ApplicationValue));
                assert!(financial_balance + application_balance >= amount, "not enough balance");
                if let Some(new_financial_balance) = financial_balance.checked_sub(amount) {
                    account.withdraw(contract_id, &TokenSource::FinancialValue, amount);
                    self.total_supply.withdraw(contract_id, &TokenSource::FinancialValue, amount);
                } else {
                    account.withdraw(contract_id, &TokenSource::FinancialValue, financial_balance);
                    self.total_supply.withdraw(contract_id, &TokenSource::FinancialValue, amount);
                    if let Some(new_application_balance) = application_balance.checked_sub(amount - financial_balance) {
                        account.withdraw(contract_id, &TokenSource::ApplicationValue, new_application_balance);
                        self.total_supply.withdraw(contract_id, &TokenSource::ApplicationValue, amount);
                    }
                }
            }
            
        }

        FtBurn {
            owner_id: account_id,
            amount: &amount.into(),
            memo: Some(&json!(contract_id).to_string()),
        }
        .emit();
    }

    

    fn contract_withdraw(&mut self, account_id: &AccountId, amount: Balance, contract_id: &AccountId, deposit_contract_id: &Option<AccountId>, token_source: &Option<TokenSource>) -> Vec<(AccountId, TokenSource, u128)> {
        let mut account = self.accounts.get(&account_id).expect(format!("The account {} is not registered", &account_id.to_string()).as_str());
        let mut used_amount = (0, 0);
        let mut total_withdraw = vec![];
        match token_source {
            Some(token_source) => {
                let deposit_balance = account.get_deposit_balance(contract_id, &Some(token_source.clone()));
                assert!(deposit_balance >= amount, "not enough balance");
                let withdraw = account.contract_withdraw(contract_id, deposit_contract_id, &TokenSource::FinancialValue, amount);
                total_withdraw = [total_withdraw, withdraw].concat();
            },
            None => {
                let financial_balance = account.get_balance(&Some(contract_id.clone()), &Some(TokenSource::FinancialValue));
                let application_balance = account.get_balance(&Some(contract_id.clone()), &Some(TokenSource::ApplicationValue));
                assert!(financial_balance + application_balance >= amount, "not enough balance");
                if let Some(new_financial_balance) = financial_balance.checked_sub(amount) {
                    let withdraw = account.contract_withdraw(contract_id, deposit_contract_id, &TokenSource::FinancialValue, amount);
                    total_withdraw = [total_withdraw, withdraw].concat();
                } else {
                    let withdraw = account.contract_withdraw(contract_id, deposit_contract_id,&TokenSource::FinancialValue, financial_balance);
                    total_withdraw = [total_withdraw, withdraw].concat();
                    if let Some(new_application_balance) = application_balance.checked_sub(amount - financial_balance) {
                        let withdraw = account.contract_withdraw(contract_id, deposit_contract_id, &TokenSource::ApplicationValue, new_application_balance);
                        total_withdraw = [total_withdraw, withdraw].concat();
                    }
                }
            }
            
        }
        total_withdraw
    }


    pub fn internal_register_account(&mut self, account_id: &AccountId) {
        if self.accounts.insert(account_id, &Account::new(account_id.to_string())).is_some() {
            env::panic_str("The account is already registered");
        }
    }
    
}

impl FungibleTokenCore for FungibleToken {

    fn ft_deposit_call(
        &mut self,
        receiver_id: AccountId,
        contract_id: AccountId,
        token_source: Option<TokenSource>,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert!(env::attached_deposit() >= 1, "not enough deposit");
        require!(env::prepaid_gas() > GAS_FOR_FT_DEPOSIT_CALL, "More gas is required");

        let sender_id = env::predecessor_account_id();

        let account = self.accounts.get(&sender_id).expect(format!("The account {} is not registered", &sender_id.to_string()).as_str());
        assert!(account.get_balance(&Some(contract_id.clone()), &token_source) >= amount.0, "not enough balance");

        self.internal_contract_deposit(&sender_id, amount.0, &contract_id, &receiver_id, &token_source);

        ext_ft_receiver::ext(receiver_id.clone())
        .with_static_gas(env::prepaid_gas() - GAS_FOR_FT_DEPOSIT_CALL)
        .with_attached_deposit(env::attached_deposit())
        .ft_on_deposit(sender_id.clone(), contract_id.clone(), token_source.clone(), amount.into(), msg)
        .then(
            ext_ft_resolver::ext(env::current_account_id())
                .with_static_gas(GAS_FOR_RESOLVE_BURN)
                .ft_resolve_deposit(sender_id, receiver_id, contract_id, token_source, amount),
        )
        .into()
    }

    fn ft_burn_call(
        &mut self,
        receiver_id: AccountId,
        contract_id: AccountId,
        token_source: Option<TokenSource>,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert!(env::attached_deposit() >= 1, "not enough deposit");
        require!(env::prepaid_gas() > GAS_FOR_FT_BURN_CALL, "More gas is required");
        let sender_id = env::predecessor_account_id();
        let account = self.accounts.get(&sender_id).expect(format!("The account {} is not registered", &sender_id.to_string()).as_str());
        assert!(account.get_balance(&Some(contract_id.clone()), &token_source) >= amount.0, "not enough balance");

        ext_ft_receiver::ext(receiver_id.clone())
        .with_static_gas(env::prepaid_gas() - GAS_FOR_FT_BURN_CALL)
        .with_attached_deposit(env::attached_deposit())
        .ft_on_burn(sender_id.clone(), contract_id.clone(), token_source.clone(), amount.into(), msg)
        .then(
            ext_ft_resolver::ext(env::current_account_id())
                .with_static_gas(GAS_FOR_RESOLVE_BURN)
                .ft_resolve_burn(sender_id, contract_id, token_source, amount),
        )
        .into()
    }

    fn ft_total_supply(&self, contract_id: Option<AccountId>, token_source: Option<TokenSource>) -> U128 {
        self.total_supply.get_balance(&contract_id, &token_source).into()
    }

    fn ft_balance_of(&self, account_id: AccountId, contract_id: Option<AccountId>, token_source: Option<TokenSource>) -> U128 {
        match self.accounts.get(&account_id) {
            Some(account) => account.get_balance(&contract_id, &token_source).into(),
            None => 0.into()
        }
    }
}

impl FungibleToken {

    pub fn internal_ft_resolve_deposit(
        &mut self,
        owner_id: &AccountId,
        receiver_id: &AccountId,
        contract_id: &AccountId,
        token_source: Option<TokenSource>,
        amount: u128,
    ) -> u128 {

        // Get the unused amount from the `ft_on_transfer` call result.
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

        if used_amount > 0 {
            self.internal_contract_deposit(owner_id, amount, &contract_id, receiver_id, &token_source);
            return used_amount;
        }
        0
    }


    /// Internal method that returns the amount of burned tokens in a corner case when the sender
    /// has deleted (unregistered) their account while the `ft_transfer_call` was still in flight.
    /// Returns (Used token amount, Burned token amount)
    pub fn internal_ft_resolve_burn(
        &mut self,
        owner_id: &AccountId,
        contract_id: &AccountId,
        token_source: Option<TokenSource>,
        amount: u128,
    ) -> u128 {

        // Get the unused amount from the `ft_on_transfer` call result.
        let used_amount: Balance = match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(value) => {
                if let Ok(unused_amount) = near_sdk::serde_json::from_slice::<U128>(&value) {
                    if let Some(unused_amount) = amount.checked_sub(unused_amount.0) {
                        amount - unused_amount
                    } else {
                        amount
                    }
                } else {
                    0
                }
            }
            PromiseResult::Failed => 0,
        };

        if used_amount > 0 {
            let contract_ids = self.wrapped_withdraw(owner_id, used_amount, &contract_id, &token_source);
            for (contract_id, token_source, withdraw_amount) in contract_ids {
                ext_ft_receiver::ext(contract_id.clone())
                .with_unused_gas_weight(1)
                .ft_on_withdraw(owner_id.clone(), contract_id, Some(token_source), withdraw_amount.into());
            }
            return used_amount;
        }
        0
    }
}

impl FungibleTokenResolver for FungibleToken {
    fn ft_resolve_deposit(
        &mut self,
        owner_id: AccountId,
        receiver_id: AccountId,
        contract_id: AccountId,
        token_source: Option<TokenSource>,
        amount: U128,
    ) -> U128 {
        self.internal_ft_resolve_deposit(&owner_id, &receiver_id, &contract_id, token_source, amount.0).into()
    }

    fn ft_resolve_burn(
        &mut self,
        owner_id: AccountId,
        contract_id: AccountId,
        token_source: Option<TokenSource>,
        amount: U128,
    ) -> U128 {
        self.internal_ft_resolve_burn(&owner_id, &contract_id, token_source, amount.0).into()
    }

}
