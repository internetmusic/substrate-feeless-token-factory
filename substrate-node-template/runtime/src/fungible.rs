/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references


/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs

use support::{decl_module, decl_storage, decl_event, ensure,
	Parameter, StorageValue, StorageMap, dispatch::Result
};
use sr_primitives::traits::{Member, SimpleArithmetic, Zero, StaticLookup, One, CheckedAdd, CheckedSub};
use sr_primitives::weights::SimpleDispatchInfo;
use system::ensure_signed;

pub trait Trait: system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	/// The units in which we record balances.
	type TokenBalance: Member + Parameter + SimpleArithmetic + Default + Copy;

	/// The arithmetic type of asset identifier.
	type TokenId: Parameter + SimpleArithmetic + Default + Copy;
}

decl_event!(
	pub enum Event<T> where
		AccountId = <T as system::Trait>::AccountId,
		TokenId = <T as Trait>::TokenId,
		TokenBalance = <T as Trait>::TokenBalance,
	{
		NewToken(TokenId, AccountId, TokenBalance),
		Transfer(TokenId, AccountId, AccountId, TokenBalance),
		Approval(TokenId, AccountId, AccountId, TokenBalance),
	}
);

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as Fungible {
		Count get(count): T::TokenId;
		TotalSupply get(total_supply): map T::TokenId => T::TokenBalance;
		Balances get(balance_of): map (T::TokenId, T::AccountId) => T::TokenBalance;
		Allowance get(allowance_of): map (T::TokenId, T::AccountId, T::AccountId) => T::TokenBalance;
	}
}

// The module's dispatchable functions.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event<T>() = default;

		#[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
		fn create_token(origin, #[compact] total_supply: T::TokenBalance) {
			let sender = ensure_signed(origin)?;

			let id = Self::count();
			let next_id = id.checked_add(&One::one()).ok_or("overflow when adding new token")?;

			<Balances<T>>::insert((id, sender.clone()), total_supply);
			<TotalSupply<T>>::insert(id, total_supply);
			<Count<T>>::put(next_id);

			Self::deposit_event(RawEvent::NewToken(id, sender, total_supply));
		}

		fn transfer(origin,
			#[compact] id: T::TokenId,
			to: <T::Lookup as StaticLookup>::Source,
			#[compact] amount: T::TokenBalance
		) {
			let sender = ensure_signed(origin)?;
			let to = T::Lookup::lookup(to)?;

			Self::make_transfer(id, sender, to, amount)?;
		}

		fn approve(origin,
			#[compact] id: T::TokenId,
			spender: <T::Lookup as StaticLookup>::Source,
			#[compact] value: T::TokenBalance
		) {
			let sender = ensure_signed(origin)?;
			let spender = T::Lookup::lookup(spender)?;

			<Allowance<T>>::insert((id, sender.clone(), spender.clone()), value);
			
			Self::deposit_event(RawEvent::Approval(id, sender, spender, value));
		}

		fn transfer_from(origin,
			#[compact] id: T::TokenId,
			from: T::AccountId,
			to: T::AccountId,
			#[compact] value: T::TokenBalance
		) {
			let sender = ensure_signed(origin)?;
			let allowance = Self::allowance_of((id, from.clone(), sender.clone()));

			let updated_allowance = allowance.checked_sub(&value).ok_or("underflow in calculating allowance")?;

			Self::make_transfer(id, from.clone(), to.clone(), value)?;

			<Allowance<T>>::insert((id, from, sender), updated_allowance);
		}
	}
}

impl<T: Trait> Module<T> {
	fn make_transfer(id: T::TokenId, from: T::AccountId, to: T::AccountId, amount: T::TokenBalance) -> Result {
		ensure!(!amount.is_zero(), "transfer amount should be non-zero");
		
		let from_balance = Self::balance_of((id, from.clone()));
		ensure!(from_balance >= amount.clone(), "origin account balance must be greater than or equal to the transfer amount");

		<Balances<T>>::insert((id, from.clone()), from_balance - amount.clone());
		<Balances<T>>::mutate((id, to.clone()), |balance| *balance += amount.clone());

		Self::deposit_event(RawEvent::Transfer(id, from, to, amount));

		Ok(())
	}
}

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, assert_ok, parameter_types};
	use sr_primitives::{traits::{BlakeTwo256, IdentityLookup}, testing::Header};
	use sr_primitives::weights::Weight;
	use sr_primitives::Perbill;

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	}
	impl system::Trait for Test {
		type Origin = Origin;
		type Call = ();
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type WeightMultiplierUpdate = ();
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
	}
	impl Trait for Test {
		type Event = ();
	}
	type TemplateModule = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	#[test]
	fn it_works_for_default_value() {
		with_externalities(&mut new_test_ext(), || {
			// Just a dummy test for the dummy funtion `do_something`
			// calling the `do_something` function with a value 42
			assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
			// asserting that the stored value is equal to what we stored
			assert_eq!(TemplateModule::something(), Some(42));
		});
	}
}