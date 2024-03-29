use near_sdk::{ext_contract, json_types::U128, AccountId};

#[ext_contract(ext_ft_resolver)]
pub trait FungibleTokenResolver {

    fn ft_resolve_deposit(
        &mut self,
        owner_id: AccountId,
        receiver_id: AccountId,
        contract_id: AccountId,
        amount: U128,
    ) -> U128;

    fn ft_resolve_withdraw(
        &mut self,
        owner_id: AccountId,
        receiver_id: AccountId,
        contract_id: AccountId,
        amount: U128,
    ) -> U128;

    fn ft_resolve_burn(
        &mut self,
        owner_id: AccountId,
        contract_id: AccountId,
        amount: U128,
    ) -> U128;
}
