
use near_sdk::Balance;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::ext_contract;
use near_sdk::json_types::U128;
use near_sdk::AccountId;
use near_sdk::PromiseOrValue;
use near_sdk::serde::{Serialize, Deserialize};


#[ext_contract(ext_ft_core)]
pub trait FungibleTokenCore {

    fn ft_available_supply(&self, contract_id: Option<AccountId>) -> U128;

    /// Returns the total supply of the token in a decimal string representation.
    fn ft_total_supply(&self, contract_id: Option<AccountId>) -> U128;

    /// Returns the balance of the account. If the account doesn't exist must returns `"0"`.
    fn ft_balance_of(&self, account_id: AccountId, contract_id: Option<AccountId>) -> U128;

    fn ft_total_balance_of(&self, account_id: AccountId, contract_id: Option<AccountId>) -> U128;
}
