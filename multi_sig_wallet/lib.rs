#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod multi_sig_wallet {
    use ink_prelude::{
        vec::Vec,
    };

    use ink_storage::{
        traits::{
            PackedLayout,
            SpreadAllocate,
            SpreadLayout,
        },
        Mapping,
    };

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InvalidTransactionId,
        InvalidOwner,
        TransactionAlreadyConfirmed,
        TransactionNotConfirmed,
        TransactionAlreadyExecuted,
        InsufficientConfirmations,
        TransactionFailed,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(event)]
    pub struct SubmitTransaction {
        owner: AccountId,
        transaction_id: TransactionId,
        to: AccountId,
        value: Balance,
    }

    #[ink(event)]
    pub struct ConfirmTransaction {
        owner: AccountId,
        transaction_id: u32,
    }

    #[ink(event)]
    pub struct RevokeConfirmation {
        owner: AccountId,
        transaction_id: u32,
    }

    type TransactionId = u32;

    #[derive(scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(
            Debug,
            PartialEq,
            Eq,
            scale_info::TypeInfo,
            ink_storage::traits::StorageLayout
        )
    )]
    pub struct Transaction {
        to: AccountId,
        value: Balance,
        executed: bool,
        num_confirmations: u32,
    }

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct MultiSigWallet {
        threshold: u32,
        owners: Mapping<AccountId, ()>,
        transactions: Mapping<TransactionId, Transaction>,
        confirmations: Mapping<(TransactionId, AccountId), ()>,
        next_transaction_id: TransactionId,
    }

    impl MultiSigWallet {
        #[ink(constructor)]
        pub fn new(mut owners: Vec<AccountId>, threshold: u32) -> Self {
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                owners.dedup();
                assert!(0 < threshold && threshold <= owners.len() as u32);

                for owner in &owners {
                    contract.owners.insert(owner, &());
                }

                contract.threshold = threshold;
                contract.transactions = Default::default();
                contract.confirmations = Default::default();
                contract.next_transaction_id = 0;
            })
        }

        #[ink(message, payable)]
        pub fn receive(&mut self) {
            ink_env::debug_println!(
                "received payment: {}",
                self.env().transferred_value()
            );
        }

        #[ink(message)]
        pub fn submit_transaction(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let caller = Self::env().caller();
            self.ensure_owner(caller)?;

            let transaction_id = self.next_transaction_id;
            self.next_transaction_id += 1;
            self.transactions.insert(transaction_id, &Transaction {
                to: to,
                value: value,
                executed: false,
                num_confirmations: 0,
            });

            Self::env().emit_event(SubmitTransaction {
                owner: caller,
                transaction_id: transaction_id,
                to: to,
                value: value,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn confirm_transaction(&mut self, transaction_id: TransactionId) -> Result<()> {
            let caller = Self::env().caller();
            self.ensure_owner(caller)?;

            let confirmation_key = (transaction_id, caller);
            if self.confirmations.get(confirmation_key).is_some() {
                return Err(Error::TransactionAlreadyConfirmed)
            };

            self.confirmations.insert(confirmation_key, &());

            let mut transaction = self.transactions.get(transaction_id)
                .ok_or(Error::InvalidTransactionId)?;
        
            if transaction.executed {
                return Err(Error::TransactionAlreadyExecuted)
            }

            transaction.num_confirmations += 1;
            self.transactions.insert(transaction_id, &transaction);


            Self::env().emit_event(ConfirmTransaction {
                owner: caller,
                transaction_id: transaction_id,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn revoke_cofirmation(&mut self, transaction_id: TransactionId) -> Result<()> {
            let caller = Self::env().caller();
            self.ensure_owner(caller)?;

            let confirmation_key = (transaction_id, caller);
            if self.confirmations.get(confirmation_key).is_none() {
                return Err(Error::TransactionNotConfirmed)
            };

            self.confirmations.remove(confirmation_key);

            let mut transaction = self.transactions.get(transaction_id)
                .ok_or(Error::InvalidTransactionId)?;
        
            if transaction.executed {
                return Err(Error::TransactionAlreadyExecuted)
            }

            transaction.num_confirmations -= 1;
            self.transactions.insert(transaction_id, &transaction);

            Self::env().emit_event(RevokeConfirmation {
                owner: caller,
                transaction_id: transaction_id,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn execute_transaction(&mut self, transaction_id: TransactionId) -> Result<()> {
            let caller = Self::env().caller();
            self.ensure_owner(caller)?;

            let mut transaction = self.transactions.get(transaction_id)
                .ok_or(Error::InvalidTransactionId)?;

            if transaction.executed {
                return Err(Error::TransactionAlreadyExecuted)
            }

            if transaction.num_confirmations < self.threshold {
                return Err(Error::InsufficientConfirmations) 
            }

            transaction.executed = true;
            self.transactions.insert(transaction_id, &transaction);

            if self.env().transfer(transaction.to, transaction.value).is_err() {
                return Err(Error::TransactionFailed)
            }

            Ok(())
        }

        #[ink(message)]
        pub fn get_transaction(&self, transaction_id: TransactionId) -> Result<Transaction> {
            let transaction = self.transactions.get(transaction_id)
                .ok_or(Error::InvalidTransactionId)?;
                
            Ok(transaction)
        }

        fn ensure_owner(&self, owner: AccountId) -> Result<()> {
            self.owners.get(owner).ok_or(Error::InvalidOwner)
        }
    }
}
