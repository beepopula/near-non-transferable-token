use near_sdk::{AccountId, Balance};

use crate::fungible_token::core::TokenSource;


pub trait FungibleTokenAccount {
    fn deposit(&mut self, contract_id: &AccountId, token_source: &TokenSource, amount: Balance);

    fn withdraw(&mut self, contract_id: &AccountId, token_source: &TokenSource, amount: Balance) -> u128;

    fn contract_deposit(&mut self, contract_id: &AccountId, deposit_contract_id: &AccountId, token_source: &TokenSource, amount: Balance);

    fn contract_withdraw(&mut self, contract_id: &AccountId, deposit_contract_id: &Option<AccountId>, token_source: &TokenSource, amount: Balance) -> Vec<(AccountId, TokenSource, u128)>;

    fn get_balance(&self, contract_id: &Option<AccountId>, token_source: &Option<TokenSource>) -> u128;

    fn get_deposit_balance(&self, contract_id: &Option<AccountId>, token_source: &Option<TokenSource>) -> u128;

    fn is_registered(&self, contract_id: &AccountId) -> bool;
}