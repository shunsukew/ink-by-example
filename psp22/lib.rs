#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[brush::contract]
mod psp22 {
    use brush::contracts::psp22::*;
    use brush::contracts::psp22::extensions::metadata::*;
    use brush::contracts::psp22::extensions::mintable::*;
    use brush::contracts::psp22::extensions::burnable::*;
    use ink_prelude::string::String;
    use ink_storage::traits::SpreadAllocate;
    
    #[ink(storage)]
    #[derive(Default, SpreadAllocate, PSP22Storage, PSP22MetadataStorage)]
    pub struct MyPSP22 {
        #[PSP22StorageField]
        psp22: PSP22Data,
        #[PSP22MetadataStorageField]
        metadata: PSP22MetadataData,
        cap: Balance,
    }

    impl PSP22 for MyPSP22 {}
    impl PSP22Metadata for MyPSP22 {}
    impl PSP22Mintable for MyPSP22 {}
    impl PSP22Burnable for MyPSP22 {}

    impl MyPSP22 {
        #[ink(constructor)]
        pub fn new(initial_supply: Balance, cap: Balance, name: Option<String>, symbol: Option<String>, decimal: u8) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                instance.metadata.name = name;
                instance.metadata.symbol = symbol;
                instance.metadata.decimals = decimal;
                assert!(instance.init_cap(cap).is_ok());
                assert!(instance._mint(instance.env().caller(), initial_supply).is_ok());
            })
        }

        #[ink(message)]
        pub fn mint(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            self._mint(account, amount)
        }

        #[ink(message)]
        pub fn cap(&self) -> Balance {
            self.cap
        }

        /// Overrides the `_mint` function to check for cap overflow before minting tokens
        /// Performs `PSP22::_mint` after the check succeeds
        fn _mint(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            if (self.total_supply() + amount) > self.cap() {
                return Err(PSP22Error::Custom(String::from("Cap exceeded")))
            }
            PSP22Internal::_mint(self, account, amount)
        }

        fn init_cap(&mut self, cap: Balance) -> Result<(), PSP22Error> {
            if cap <= 0 {
                return Err(PSP22Error::Custom(String::from("Cap must be above 0")))
            }
            self.cap = cap;
            Ok(())
        }
    }
}
