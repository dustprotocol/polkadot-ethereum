#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_support::{
	decl_module, decl_storage, decl_event, decl_error,
	weights::Weight,
	traits::Get
};

use sp_io::hashing::keccak_256;
use sp_core::{H160, H256};
use sp_runtime::{
	traits::Zero,
	DigestItem
};

use codec::{Encode, Decode};
use artemis_core::Commitments;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Custom DigestItem for header digest
#[derive(Encode, Decode, Copy, Clone)]
enum CustomDigestItem {
	Commitment(H256)
}

impl<T> Into<DigestItem<T>> for CustomDigestItem {
    fn into(self) -> DigestItem<T> {
        DigestItem::Other(self.encode())
    }
}

#[derive(Encode, Decode, Clone)]
struct Message {
	address: H160,
	payload: Vec<u8>,
}

pub trait Trait: frame_system::Trait {
	type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;

	type PruneInterval: Get<Self::BlockNumber>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Commitments {
		/// messages for an application
		Messages: Vec<Message>
	}
}

decl_event! {
	pub enum Event {}
}

decl_error! {
	pub enum Error for Module<T: Trait> {}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		// Generate a message commitment every `T::PruneInterval` blocks
		fn on_initialize(now: T::BlockNumber) -> Weight {
			if (now % T::PruneInterval::get()).is_zero() {
				Self::commit()
			} else {
				0
			}
		}

	}
}

impl<T: Trait> Module<T> {

	// Generate a message commitment
	// TODO: return proper weight
	fn commit() -> Weight {
		let mut digest = <frame_system::Module<T>>::digest();
		let messages: Vec<Message> = <Self as Store>::Messages::get();

		let hash: H256 = keccak_256(messages.encode().as_ref()).into();
		digest.push(CustomDigestItem::Commitment(hash).into());

		// Prune messages
		<Self as Store>::Messages::kill();

		0
	}
}

impl<T: Trait> Commitments for Module<T> {

	// Add a message for eventual inclusion in a commitment
	fn add(address: H160, payload: Vec<u8>) {
		<Self as Store>::Messages::append(Message { address, payload });
	}
}
