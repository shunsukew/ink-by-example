#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod hello_world {
    use ink_prelude::string::String;
    
    #[ink(storage)]
    pub struct HelloWorld {}

    impl HelloWorld {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn hello_world(&self) -> String {
            "Hello World".into()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn it_works() {
            let hello_world = HelloWorld::new();
            assert_eq!(hello_world.hello_world(), "Hello World");
        }
    }
}
