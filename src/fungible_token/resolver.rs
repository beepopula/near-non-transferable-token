use near_sdk::{ext_contract, json_types::U128, AccountId};
use crate::*;

#[ext_contract(ext_ft_resolver)]
pub trait FungibleTokenResolver {
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128;

    fn ft_resolve_burn(
        &mut self,
        owner_id: AccountId,
        amount: U128,
        contract_id: AccountId,
        token_dest: TokenDest
    ) -> U128;
}
