
use near_sdk::Balance;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::ext_contract;
use near_sdk::json_types::U128;
use near_sdk::AccountId;
use near_sdk::PromiseOrValue;
use near_sdk::serde::{Serialize, Deserialize};

// #[derive(BorshDeserialize, BorshSerialize, PartialOrd, PartialEq, Eq, Hash)]
// #[derive(Serialize, Deserialize, Clone)]
// #[serde(crate = "near_sdk::serde")]
// pub enum TokenSource {
//     ApplicationValue,
//     FinancialValue
// }




#[ext_contract(ext_ft_core)]
pub trait FungibleTokenCore {

    /// Returns the total supply of the token in a decimal string representation.
    fn ft_total_supply(&self, contract_id: Option<AccountId>) -> U128;

    /// Returns the balance of the account. If the account doesn't exist must returns `"0"`.
    fn ft_balance_of(&self, account_id: AccountId, contract_id: Option<AccountId>) -> U128;
}
