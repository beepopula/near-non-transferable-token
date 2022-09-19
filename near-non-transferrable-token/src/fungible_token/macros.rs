/// The core methods for a basic fungible token. Extension standards may be
/// added in addition to this macro.
#[macro_export]
macro_rules! impl_fungible_token_core {
    ($contract: ident, $token: ident $(, $on_tokens_burned_fn:ident)?) => {

        #[near_bindgen]
        impl FungibleTokenCore for $contract {
            #[payable]
            fn ft_transfer(
                &mut self,
                receiver_id: AccountId,
                amount: U128,
                memo: Option<String>,
            ) {
                self.$token.ft_transfer(receiver_id, amount, memo)
            }

            #[payable]
            fn ft_transfer_call(
                &mut self,
                receiver_id: AccountId,
                amount: U128,
                memo: Option<String>,
                msg: String,
            ) -> PromiseOrValue<U128> {
                self.$token.ft_transfer_call(receiver_id, amount, memo, msg)
            }

            fn ft_total_supply(&self, contract_id: Option<AccountId>, token_source: Option<TokenSource>) -> U128 {
                self.$token.ft_total_supply(contract_id, token_source)
            }

            fn ft_balance_of(&self, account_id: AccountId, contract_id: Option<AccountId>, token_source: Option<TokenSource>) -> U128 {
                self.$token.ft_balance_of(account_id, contract_id, token_source)
            }

            fn ft_burn_call(
                &mut self,
                contract_id: AccountId, 
                token_dest: TokenDest,
                amount: U128, 
                msg: String
            ) -> PromiseOrValue<U128> {
                self.$token.ft_burn_call(contract_id, token_dest, amount, msg)
            }
        }

        #[near_bindgen]
        impl FungibleTokenResolver for $contract {
            #[private]
            fn ft_resolve_deposit(
                &mut self,
                sender_id: AccountId,
                receiver_id: AccountId,
                amount: U128,
            ) -> U128 {
                self.$token.ft_resolve_deposit(sender_id, receiver_id, amount).into()
            }

            #[private]
            fn ft_resolve_burn(
                &mut self,
                owner_id: AccountId, 
                amount: U128, 
                contract_id: AccountId,
                token_dest: TokenDest
            ) -> U128 {
                self.$token.ft_resolve_burn(owner_id, amount, contract_id, token_dest)
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

            #[payable]
            fn storage_unregister(&mut self, force: Option<bool>) -> bool {
                #[allow(unused_variables)]
                if let Some((account_id, balance)) = self.$token.internal_storage_unregister(force) {
                    $(self.$on_account_closed_fn(account_id, balance);)?
                    true
                } else {
                    false
                }
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
