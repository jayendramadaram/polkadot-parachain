#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    traits::Get,
};
use frame_system::ensure_signed;
use sp_runtime::AccountId32;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

type AccountId<T> = <T as frame_system::Config>::AccountId;


#[derive(PartialEq, Eq, Clone)]
enum Chain {
    Bitcoin,
    Ethereum,
}

enum Status {
    OrderCreated,
	OrderFilled,
	InitiatorAtomicSwapInitiated,
	FollowerAtomicSwapInitiated,
	FollowerAtomicSwapRedeemed,
	InitiatorAtomicSwapRedeemed,
	InitiatorAtomicSwapRefunded,
	FollowerAtomicSwapRefunded,
	OrderExecuted,
	OrderFailedSoft,
	OrderFailedHard,
}

#[derive(PartialEq, Eq, Clone, Default)]
pub struct Order<AccountId> {
    creator: AccountId,
    filler: Option<AccountId>,
    InitatorSwap : AtomicSwap,
    RedeemerSwap : AtomicSwap,
    SecretHash: String,
    status : Status
}

#[derive(PartialEq, Eq, Clone, Default)]
pub struct AtomicSwap {
    Amount: u64,
    address : String,
    chain : Chain,
    initx: String,
    redeemtx: String,
    refundtx: String,
}

decl_storage! {
    trait Store for Module<T: Config> as TemplateModule {
        OrderMap get(fn order_map): map hasher(blake2_128_concat) u64 => Order<AccountId<T>>;
        NextOrderId get(fn next_order_id): u64;
    }
}

decl_event! {
    pub enum Event<T> where AccountId = AccountId<T> {
        OrderCreated(u64, AccountId, String),
        OrderFilled(u64, AccountId),
        OrderCancelled(u64, AccountId),
        OrderUpdated(u64, AccountId, String),
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        OrderNotFound,
        OrderAlreadyFilled,
        NotOrderCreator,
        InvalidUpdateMsg,
        VerificationFailed,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        #[pallet::weight(10_000)]
        pub fn create_order(origin, sendAmout u64,reciveAmout u64,fromChain Chain,toChain Chain ,) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let order_id = Self::next_order_id();
            let order = Order {
                creator: sender.clone(),
                filler: None,
                InitatorSwap : AtomicSwap {
                    Amount: sendAmout,
                    address : String::from(""),
                    chain : fromChain,
                    initx: String::from(""),
                    redeemtx: String::from(""),
                    refundtx: String::from(""),
                },
                RedeemerSwap : AtomicSwap {
                    Amount: reciveAmout,
                    address : String::from(""),
                    chain : toChain,
                    initx: String::from(""),
                    redeemtx: String::from(""),
                    refundtx: String::from(""),
                },
                SecretHash: String::from(""),
            };

            OrderMap::<T>::insert(order_id, order.clone());
            NextOrderId::mutate(|id| *id += 1);

            Self::deposit_event(Event::<T>::OrderCreated(order_id, sender, updatemsg));
            Ok(())
        }

        #[weight = 10_000]
        pub fn fill_order(origin, order_id: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            ensure!(OrderMap::<T>::contains_key(order_id), Error::<T>::OrderNotFound);
            let mut order = OrderMap::<T>::get(order_id);
            ensure!(order.filler.is_none(), Error::<T>::OrderAlreadyFilled);

            order.filler = Some(sender.clone());
            OrderMap::<T>::insert(order_id, order.clone());

            Self::deposit_event(Event::<T>::OrderFilled(order_id, sender));
            Ok(())
        }

        #[weight = 10_000]
        pub fn cancel_order(origin, order_id: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            ensure!(OrderMap::<T>::contains_key(order_id), Error::<T>::OrderNotFound);
            let order = OrderMap::<T>::get(order_id);

            ensure!(order.creator == sender, Error::<T>::NotOrderCreator);
            ensure!(order.filler.is_none(), Error::<T>::OrderAlreadyFilled);

            OrderMap::<T>::remove(order_id);

            Self::deposit_event(Event::<T>::OrderCancelled(order_id, sender));
            Ok(())
        }

        #[weight = 10_000]
        pub fn update_order(origin, order_id: u64, verification_result: bool) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            ensure!(OrderMap::<T>::contains_key(order_id), Error::<T>::OrderNotFound);
            let mut order = OrderMap::<T>::get(order_id);

            ensure!(order.creator == sender, Error::<T>::NotOrderCreator);

            // Run the verification logic
            if verification_result {
                // Update the order's updatemsg
                let updated_msg = Self::fetch_updated_msg_from_api()?;
                order.updatemsg = updated_msg;
                OrderMap::<T>::insert(order_id, order.clone());

                Self::deposit_event(Event::<T>::OrderUpdated(order_id, sender, updated_msg));
                Ok(())
            } else {
                Err(Error::<T>::VerificationFailed.into())
            }
        }

        fn fetch_updated_msg_from_api() -> Result<String, Error<T>> {
            // Implement your API call logic here to fetch the updated message
            // and perform any necessary verification
            // Return the updated message or an error if verification fails
            // For demonstration purposes, a mock implementation is provided below

            // Mock implementation
            Ok(String::from("Updated message from API"))
        }
    }
}
