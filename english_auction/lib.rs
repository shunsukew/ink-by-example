#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[brush::contract]
mod english_auction {
    use ink_storage::{
        traits::SpreadAllocate,
        Mapping,
    };
    use brush::contracts::traits::psp34::{
        extensions::metadata::*,
        *,
    };
    use ink_prelude::{
        vec::Vec,
    };

    #[brush::wrapper]
    pub type PSP34Ref = dyn PSP34 + PSP34Metadata;

    type TokenId = u32;

    const BLACKHOLE_ACCOUNT_ID: [u8; 32] = [0; 32];

    /// Errors that can occur upon calling this contract.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        InvalidSeller,
        InvalidBidder,
        AlreadyStarted,
        NotStarted,
        AlreadyEnded,
        NotEnded,
        PSP34TransferFailed,
        InsufficientBid,
        NoWithdrawableBalance,
        TransferFailed,
    }

    /// Type alias for the contract's result type.
    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(event)]
    pub struct Bid {
        bidder: AccountId,
        amount: Balance,
    }

    #[ink(event)]
    pub struct End {
        highest_bidder: AccountId,
        highest_bid: Balance,
    }


    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct EnglishAuction {
        psp34_contract: AccountId,
        token_id: TokenId,
        seller: AccountId,
        started: bool,
        end_at: u64,
        ended: bool,

        highest_bidder: AccountId,
        highest_bid: Balance,
        bids: Mapping<AccountId, Balance>,
    }

    impl EnglishAuction {
        #[ink(constructor)]
        pub fn new(contract: AccountId, token_id: TokenId) -> Self {
            ink_lang::utils::initialize_contract(|instance: &mut Self| {
                let caller = instance.env().caller();
                instance.psp34_contract = contract;
                instance.token_id = token_id;
                instance.seller = caller;
                instance.started = false;
                instance.ended = false;
                instance.highest_bid = 0;
                instance.highest_bidder = BLACKHOLE_ACCOUNT_ID.into();
                instance.bids = Default::default();
            })
        }

        #[ink(message)]
        pub fn start(&mut self) -> Result<()> {
            let caller = self.env().caller();
            self.ensure_seller(caller)?;

            if self.started {
                return Err(Error::AlreadyStarted)
            }

            // Deposit token to the contract
            PSP34Ref::transfer(&self.psp34_contract, self.psp34_contract, Id::U32(self.token_id), Vec::new())
                .map_err(|_| Error::PSP34TransferFailed)?;

            self.started = true;

            // 10 mins for testing purpose
            self.end_at = self.env().block_timestamp() + 600;

            Ok(())
        }

        #[ink(message, payable)]
        pub fn bid(&mut self) -> Result<()> {
            let caller = self.env().caller();
            self.ensure_not_seller(caller)?;

            if !self.started {
                return Err(Error::NotStarted)
            }

            if self.ended || self.end_at <= self.env().block_timestamp() {
                return Err(Error::AlreadyEnded)
            }

            // new bid needs to be higher amount than current one
            let bid_amount: Balance = self.env().transferred_value();
            if bid_amount <= self.highest_bid {
                return Err(Error::InsufficientBid)
            }

            // reserve amont for withdrawal
            if self.highest_bidder != BLACKHOLE_ACCOUNT_ID.into() {
                let value = self.bids.get(self.highest_bidder).unwrap_or_default();
                self.bids.insert(self.highest_bidder, &(value + self.highest_bid));
            }

            self.highest_bid = bid_amount;
            self.highest_bidder = caller;

            Self::env().emit_event(Bid {
                bidder: caller,
                amount: bid_amount,
            });

            Ok(())
        }

        #[ink(message, payable)]
        pub fn end(&mut self) -> Result<()> {
            let caller = self.env().caller();
            self.ensure_seller(caller)?;

            if !self.started {
                return Err(Error::NotStarted)
            }

            if self.ended {
                return Err(Error::AlreadyEnded)
            }

            if self.env().block_timestamp() < self.end_at {
                return Err(Error::NotEnded)
            }

            self.ended = true;
            if self.highest_bidder != BLACKHOLE_ACCOUNT_ID.into() {
                PSP34Ref::transfer(&self.psp34_contract, self.seller, Id::U32(self.token_id), Vec::new())
                    .map_err(|_| Error::PSP34TransferFailed)?;
            } else {
                PSP34Ref::transfer(&self.psp34_contract, self.highest_bidder, Id::U32(self.token_id), Vec::new())
                    .map_err(|_| Error::PSP34TransferFailed)?;
            }

            Self::env().emit_event(End {
                highest_bidder: self.highest_bidder,
                highest_bid: self.highest_bid,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn withdraw(&mut self) -> Result<()> {
            let caller: AccountId = self.env().caller();
            let withdrawable_amount = self.bids.get(caller)
                .ok_or::<Error>(Error::NoWithdrawableBalance)?;
            self.bids.remove(caller);

            if self.env().transfer(caller, withdrawable_amount).is_err() {
                return Err(Error::TransferFailed)
            }

            Ok(())
        }

        fn ensure_seller(&self, caller: AccountId) -> Result<()> {
            if self.seller != caller {
                return Err(Error::InvalidSeller)
            }
            Ok(())
        }

        fn ensure_not_seller(&self, caller: AccountId) -> Result<()> {
            if self.seller == caller {
                return Err(Error::InvalidBidder)
            }
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use ink_lang as ink;

        #[ink::test]
        #[ink::test]
        fn it_works() {
        }
    }
}
