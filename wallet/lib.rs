#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod wallet {
    #[ink(storage)]
    pub struct Wallet {
        owner: AccountId,
    }

    impl Wallet {
        #[ink(constructor)]
        pub fn new(owner: AccountId) -> Self {
            Self { owner }
        }

        #[ink(message)]
        pub fn get_owner(&self) -> AccountId {
            self.owner
        }

        #[ink(message, payable)]
        pub fn receive(&mut self) {
            ink_env::debug_println!(
                "received payment: {}",
                self.env().transferred_value()
            );
        }

        #[ink(message)]
        pub fn withdraw(&mut self, amount: Balance) {
            ink_env::debug_println!("requested amount: {}", amount);
            ink_env::debug_println!("contract balance: {}", self.env().balance());

            assert!(self.env().caller() == self.owner, "only owner can withdraw funds");
            assert!(amount <= self.env().balance(), "insufficient funds");

            if self.env().transfer(self.env().caller(), amount).is_err() {
                panic!(
                    "requested transfer failed. this can be the case if the contract does not\
                     have sufficient free funds or if the transfer would have brought the\
                     contract's balance below minimum balance."
                )
            }
        }

        #[ink(message)]
        pub fn get_balance(&self) -> Balance {
            self.env().balance()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use ink_lang as ink;

        #[ink::test]
        fn test_constructor_works() {
            let accounts = default_accounts();
            let wallet = Wallet::new(accounts.alice);
            assert_eq!(wallet.get_owner(), accounts.alice);
        }

        #[ink::test]
        fn test_get_balance_works() {
            let wallet = create_contract(100);
            assert_eq!(wallet.get_balance(), 100);
        }

        #[ink::test]
        #[should_panic(expected = "insufficient funds")]
        fn test_withdraw_fails_insufficent_balance() {
            let accounts = default_accounts();
            
            let mut wallet = create_contract(100);

            set_sender(accounts.alice);
            wallet.withdraw(1000);
        }

        #[ink::test]
        #[should_panic(expected = "only owner can withdraw funds")]
        fn test_withdraw_fails_not_owner() {
            let accounts = default_accounts();
            
            let mut wallet = create_contract(100);

            set_sender(accounts.eve);
            wallet.withdraw(10);
        }

        #[ink::test]
        fn test_withdraw_works() {
            let accounts = default_accounts();
            let mut wallet = create_contract(100);

            set_sender(accounts.alice);

            let former_balance = get_balance(accounts.alice);
            wallet.withdraw(10);
            let current_balance = get_balance(accounts.alice);

            assert!(current_balance - former_balance == 10);
        }

        #[ink::test]
        fn test_recieve_works() {
            let accounts = default_accounts();
            let mut wallet = create_contract(100);

            set_sender(accounts.eve);
            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(10);
            wallet.receive();
        }

        fn create_contract(initial_balance: Balance) -> Wallet {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_balance(contract_id(), initial_balance);
            Wallet::new(accounts.alice)
        }

        fn contract_id() -> AccountId {
            ink_env::test::callee::<ink_env::DefaultEnvironment>()
        }

        fn set_sender(sender: AccountId) {
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(sender);
        }

        fn default_accounts(
        ) -> ink_env::test::DefaultAccounts<ink_env::DefaultEnvironment> {
            ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
        }

        fn set_balance(account_id: AccountId, balance: Balance) {
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
                account_id, balance,
            )
        }

        fn get_balance(account_id: AccountId) -> Balance {
            ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(account_id)
                .expect("Cannot get account balance")
        }
    }
}
