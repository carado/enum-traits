#![feature(
)]

pub use ::enum_traits_proc_macros::DiscriminantValues;

pub unsafe trait DiscriminantValues {
	type Discriminant: 'static + Clone + Copy + std::fmt::Debug + Eq
		+ PartialEq<Self::Discriminant> + std::hash::Hash + Send + Sync + Unpin;
	
	const VALUES: &'static [Self::Discriminant];
}

pub unsafe trait ReprC: DiscriminantValues {
	fn discriminant(&self) -> &Self::Discriminant {
		unsafe { &*(self as *const _ as *const _) }
	}
}


