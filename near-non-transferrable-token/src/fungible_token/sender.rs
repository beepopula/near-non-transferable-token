use near_sdk::{AccountId, json_types::U128, PromiseOrValue};



pub trait FungibleTokenSender {
    
    fn ft_deposit_call(
        &mut self,
        receiver_id: AccountId,
        contract_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;

    fn ft_withdraw_call(
        &mut self,
        receiver_id: AccountId,
        contract_id: AccountId,
        amount: U128,
    ) -> PromiseOrValue<U128>;

    fn ft_burn_call(
        &mut self,
        receiver_id: AccountId,
        contract_id: AccountId,
        amount: U128,
        msg: String
    ) -> PromiseOrValue<U128>;
}
