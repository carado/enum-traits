#![feature(
)]

pub use ::enum_traits_proc_macros::DiscriminantValues;

pub unsafe trait DiscriminantValues {
	type Discriminant: 'static + Clone + Copy + std::fmt::Debug + Eq
		+ PartialEq<Self::Discriminant> + std::hash::Hash + Send + Sync + Unpin;
	
	const VALUES: &'static [Self::Discriminant];
}



