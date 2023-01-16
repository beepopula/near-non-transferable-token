use near_sdk::{AccountId, Balance};


pub trait FungibleTokenAccount {
    fn deposit(&mut self, contract_id: &AccountId, amount: Balance);

    fn withdraw(&mut self, contract_id: &AccountId, amount: Balance) -> u128;

    fn contract_deposit(&mut self, contract_id: &AccountId, deposit_contract_id: &AccountId, amount: Balance);

    fn contract_withdraw(&mut self, contract_id: &AccountId, deposit_contract_id: &AccountId, amount: Balance);

    fn get_available_balance(&self, contract_id: &Option<AccountId>) -> u128;

    fn get_deposit_balance(&self, contract_id: &Option<AccountId>, deposit_contract_id: &Option<AccountId>) -> u128;

    fn get_total_balance(&self, contract_id: &Option<AccountId>) -> u128;

    fn is_registered(&self, contract_id: &AccountId) -> bool;

    fn is_deposit_exist(&self, cotnract_id: &AccountId, deposit_contract_id: &AccountId) -> bool;
}