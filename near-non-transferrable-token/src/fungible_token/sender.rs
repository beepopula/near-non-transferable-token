use near_sdk::{AccountId, json_types::U128, PromiseOrValue};

use crate::fungible_token::core::TokenSource;



pub trait FungibleTokenSender {
    
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
}
