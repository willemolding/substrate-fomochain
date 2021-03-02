#![cfg_attr(not(feature = "std"), no_std)]

use core::convert::TryInto;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure,
    sp_runtime::{traits::AccountIdConversion, ModuleId},
    traits::{Currency, ExistenceRequirement::AllowDeath, Get},
};
use frame_system::{self as system, ensure_signed};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// This 8 char ID can be converted into an account with no keys
// This system account holds the balance of the contributed funds
const PALLET_ID: ModuleId = ModuleId(*b"fomofomo");

type AccountIdOf<T> = <T as system::Config>::AccountId;
type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Currency: Currency<Self::AccountId>;
    /// How much the price increases with each round
    type PriceIncrement: Get<BalanceOf<Self>>;
    /// How many blocks in which noone buys a ticket for the game to end
    type BlocksToWin: Get<Self::BlockNumber>;
}

decl_storage! {
    trait Store for Module<T: Config> as FOMOModule {
        Round get(fn round): u128 = 0;
        Leader get(fn leader): Option<T::AccountId>;
        LastPaymentBlock get(fn last_payment_block): T::BlockNumber = 0.into();
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        Balance = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance
    {
        /// A player purchased a ticket. [who, price]
        TicketPurchased(AccountId, Balance),
        /// The winner has claimed the pool. [who, winnings]
        PoolClaimed(AccountId, Balance),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        NoneValue,
        /// The game cannot start until a custodian account is configured
        NoCustodianSet,
        /// Attempted to purchase ticket but did not allocate enough funds in the transfer
        InsufficientFunds,
        /// An error occurs due to impls provided by runtime not supporting conversions
        ValueConversionError,
        /// The game is over preventing the intended action
        GameIsOver,
        /// The game is not over preventing the intended action
        GameIsNotOver,
        /// The account attempting to claim the pool is not the current leader
        ClaimerIsNotLeader,
    }
}

// Can add helper functions on the config here
impl<T: Config> Module<T> {
    pub fn pool_account_id() -> T::AccountId {
        PALLET_ID.into_account()
    }

    fn pool_balance() -> BalanceOf<T> {
        T::Currency::free_balance(&Self::pool_account_id())
    }

    fn current_price() -> Result<BalanceOf<T>, Error<T>> {
        let price_increment = TryInto::<u128>::try_into(T::PriceIncrement::get())
            .map_err(|_| Error::ValueConversionError)?;
        let round = Round::get();
        ((round + 1) * price_increment)
            .try_into()
            .map_err(|_| Error::ValueConversionError)
    }

    fn game_is_over() -> bool {
    	let now = <system::Module<T>>::block_number();
        let last_payment_block = LastPaymentBlock::<T>::get();
        now >= last_payment_block + T::BlocksToWin::get()
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 10_000]
        pub fn buy_ticket(origin, max_spend: BalanceOf<T>) -> dispatch::DispatchResult {
            let purchaser = ensure_signed(origin)?;
			
			// ensure game is not over
			ensure!(!Self::game_is_over(), Error::<T>::GameIsOver);

			// the caller provided enough funds to purchase a ticket
            let current_price = Self::current_price()?;
            ensure!(current_price <= max_spend, Error::<T>::InsufficientFunds);

            T::Currency::transfer(
                &purchaser,
                &Self::pool_account_id(),
                current_price,
                AllowDeath
            )?;

            // this new player is the leader and the round is incremented
            Leader::<T>::put(&purchaser);
            Round::put(Round::get() + 1);
            let now = <system::Module<T>>::block_number();
            LastPaymentBlock::<T>::put(now);

            Self::deposit_event(RawEvent::TicketPurchased(purchaser, current_price));

            Ok(())
        }


        #[weight = 10_000]
        pub fn claim(origin) -> dispatch::DispatchResult {
            let claimer = ensure_signed(origin)?;

            // ensure the claimer is the current leader
            ensure!(claimer == Leader::<T>::get().unwrap(), Error::<T>::ClaimerIsNotLeader);

            // ensure the required time to win has elapsed
            ensure!(Self::game_is_over(), Error::<T>::GameIsNotOver);

            let winning_balance = Self::pool_balance();

            // transfer all funds out of the pool
            T::Currency::transfer(
                &Self::pool_account_id(),
                &claimer,
                winning_balance,
                AllowDeath
            )?;

            Self::deposit_event(RawEvent::PoolClaimed(claimer, winning_balance));

            Ok(())
        }
    }
}
