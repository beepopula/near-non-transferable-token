/// The core methods for a basic fungible token. Extension standards may be
/// added in addition to this macro.
#[macro_export]
macro_rules! impl_fungible_token_core {
    ($contract: ident, $token: ident) => {

        #[near_bindgen]
        impl FungibleTokenCore for $contract {

            fn ft_available_supply(&self, contract_id: Option<AccountId>) -> U128 {
                self.$token.ft_available_supply(contract_id)
            }

            fn ft_total_supply(&self, contract_id: Option<AccountId>) -> U128 {
                self.$token.ft_total_supply(contract_id)
            }

            fn ft_balance_of(&self, account_id: AccountId, contract_id: Option<AccountId>) -> U128 {
                self.$token.ft_balance_of(account_id, contract_id)
            }

            fn ft_total_balance_of(&self, account_id: AccountId, contract_id: Option<AccountId>) -> U128 {
                self.$token.ft_total_balance_of(account_id, contract_id)
            }

            #[payable]
            fn ft_deposit_call(
                &mut self,
                receiver_id: AccountId,
                contract_id: AccountId,
                amount: U128,
                msg: String,
            ) -> PromiseOrValue<U128> {
                self.$token.ft_deposit_call(receiver_id, contract_id, amount, msg)
            }

            #[payable]
            fn ft_withdraw_call(
                &mut self,
                receiver_id: AccountId,
                contract_id: AccountId,
                amount: U128,
                msg: String,
            ) -> PromiseOrValue<U128> {
                self.$token.ft_withdraw_call(receiver_id, contract_id, amount, msg)
            }

            #[payable]
            fn ft_burn_call(
                &mut self,
                receiver_id: AccountId,
                contract_id: AccountId,
                amount: U128,
                msg: String,
            ) -> PromiseOrValue<U128> {
                self.$token.ft_burn_call(receiver_id, contract_id, amount, msg)
            }
        }

        #[near_bindgen]
        impl FungibleTokenResolver for $contract {

            #[private]
            fn ft_resolve_deposit(
                &mut self,
                owner_id: AccountId,
                receiver_id: AccountId,
                contract_id: AccountId,
                amount: U128,
            ) -> U128 {
                self.$token.ft_resolve_deposit(owner_id, receiver_id, contract_id, amount)
            }   
            
            #[private]
            fn ft_resolve_withdraw(
                &mut self,
                owner_id: AccountId,
                receiver_id: AccountId,
                contract_id: AccountId,
                amount: U128,
            ) -> U128 {
                self.$token.ft_resolve_withdraw(owner_id, receiver_id, contract_id, amount)
            }   

            #[private]
            fn ft_resolve_burn(
                &mut self,
                owner_id: AccountId,
                contract_id: AccountId,
                amount: U128,
            ) -> U128 {
                self.$token.ft_resolve_burn(owner_id, contract_id, amount)
            }   
        }
    };
}

/// Ensures that when fungible token storage grows by collections adding entries,
/// the storage is be paid by the caller. This ensures that storage cannot grow to a point
/// that the FT contract runs out of Ⓝ.
/// Takes name of the Contract struct, the inner field for the token and optional method name to
/// call when the account was closed.
#[macro_export]
macro_rules! impl_fungible_token_storage {
    ($contract: ident, $token: ident $(, $on_account_closed_fn:ident)?) => {

        #[near_bindgen]
        impl StorageManagement for $contract {
            #[payable]
            fn storage_deposit(
                &mut self,
                account_id: Option<AccountId>,
                registration_only: Option<bool>,
            ) -> StorageBalance {
                self.$token.storage_deposit(account_id, registration_only)
            }

            #[payable]
            fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
                self.$token.storage_withdraw(amount)
            }

            fn storage_unregister(&mut self, force: Option<bool>) -> bool {
                self.$token.storage_unregister(force)
            }

            fn storage_balance_bounds(&self) -> StorageBalanceBounds {
                self.$token.storage_balance_bounds()
            }

            fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
                self.$token.storage_balance_of(account_id)
            }
        }
    };
}
