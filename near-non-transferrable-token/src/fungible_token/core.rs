
use near_sdk::Balance;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::ext_contract;
use near_sdk::json_types::U128;
use near_sdk::AccountId;
use near_sdk::PromiseOrValue;
use near_sdk::serde::{Serialize, Deserialize};

#[derive(BorshDeserialize, BorshSerialize, PartialOrd, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum TokenSource {
    ApplicationValue,
    FinancialValue
}

pub trait FungibleTokenAccount {
    fn deposit(&mut self, contract_id: &AccountId, token_source: &TokenSource, amount: Balance);

    fn withdraw(&mut self, contract_id: &AccountId, token_source: &TokenSource, amount: Balance) -> u128;

    fn contract_deposit(&mut self, contract_id: &AccountId, deposit_contract_id: &AccountId, token_source: &TokenSource, amount: Balance);

    fn contract_withdraw(&mut self, contract_id: &AccountId, deposit_contract_id: &Option<AccountId>, token_source: &TokenSource, amount: Balance) -> Vec<(AccountId, TokenSource, u128)>;

    fn get_balance(&self, contract_id: &Option<AccountId>, token_source: &Option<TokenSource>) -> u128;

    fn get_deposit_balance(&self, contract_id: &AccountId, token_source: &Option<TokenSource>) -> u128;

    fn is_registered(&self, contract_id: &AccountId) -> bool;
}




#[ext_contract(ext_ft_core)]
pub trait FungibleTokenCore {
    /// Transfers positive `amount` of tokens from the `env::predecessor_account_id` to `receiver_id`.
    /// Both accounts must be registered with the contract for transfer to succeed. (See [NEP-145](https://github.com/near/NEPs/discussions/145))
    /// This method must to be able to accept attached deposits, and must not panic on attached deposit.
    /// Exactly 1 yoctoNEAR must be attached.
    /// See [the Security section](https://github.com/near/NEPs/issues/141#user-content-security) of the standard.
    ///
    /// Arguments:
    /// - `receiver_id` - the account ID of the receiver.
    /// - `amount` - the amount of tokens to transfer. Must be a positive number in decimal string representation.
    /// - `memo` - an optional string field in a free form to associate a memo with this transfer.

    /// Transfers positive `amount` of tokens from the `env::predecessor_account_id` to `receiver_id` account. Then
    /// calls `ft_on_transfer` method on `receiver_id` contract and attaches a callback to resolve this transfer.
    /// `ft_on_transfer` method must return the amount of tokens unused by the receiver contract, the remaining tokens
    /// must be refunded to the `predecessor_account_id` at the resolve transfer callback.
    ///
    /// Token contract must pass all the remaining unused gas to the `ft_on_transfer` call.
    ///
    /// Malicious or invalid behavior by the receiver's contract:
    /// - If the receiver contract promise fails or returns invalid value, the full transfer amount must be refunded.
    /// - If the receiver contract overspent the tokens, and the `receiver_id` balance is lower than the required refund
    /// amount, the remaining balance must be refunded. See [the Security section](https://github.com/near/NEPs/issues/141#user-content-security) of the standard.
    ///
    /// Both accounts must be registered with the contract for transfer to succeed. (See #145)
    /// This method must to be able to accept attached deposits, and must not panic on attached deposit. Exactly 1 yoctoNEAR must be attached. See [the Security
    /// section](https://github.com/near/NEPs/issues/141#user-content-security) of the standard.
    ///
    /// Arguments:
    /// - `receiver_id` - the account ID of the receiver contract. This contract will be called.
    /// - `amount` - the amount of tokens to transfer. Must be a positive number in a decimal string representation.
    /// - `memo` - an optional string field in a free form to associate a memo with this transfer.
    /// - `msg` - a string message that will be passed to `ft_on_transfer` contract call.
    ///
    /// Returns a promise which will result in the amount of tokens withdrawn from sender's account.
    fn ft_deposit_call(
        &mut self,
        receiver_id: AccountId,
        contract_id: AccountId,
        token_source: Option<TokenSource>,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;

    fn ft_burn_call(
        &mut self,
        receiver_id: AccountId,
        contract_id: AccountId,
        token_source: Option<TokenSource>,
        amount: U128,
        msg: String
    ) -> PromiseOrValue<U128>;

    /// Returns the total supply of the token in a decimal string representation.
    fn ft_total_supply(&self, contract_id: Option<AccountId>, token_source: Option<TokenSource>) -> U128;

    /// Returns the balance of the account. If the account doesn't exist must returns `"0"`.
    fn ft_balance_of(&self, account_id: AccountId, contract_id: Option<AccountId>, token_source: Option<TokenSource>) -> U128;
}
