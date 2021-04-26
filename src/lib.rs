#![feature(
)]

pub use ::enum_traits_proc_macros::*;

pub unsafe trait DiscriminantValues {
	type Discriminant: 'static + Clone + Copy + std::fmt::Debug + Eq
		+ PartialEq<Self::Discriminant> + std::hash::Hash + Send + Sync + Unpin;
	
	const VALUES: &'static [Self::Discriminant];

	const EVER_ENABLED_BITS: Self::Discriminant;
	const ALWAYS_ENABLED_BITS: Self::Discriminant;
}

pub unsafe trait DiscriminantHeaded: DiscriminantValues {
	fn discriminant(&self) -> &Self::Discriminant {
		unsafe { &*(self as *const _ as *const _) }
	}
}

