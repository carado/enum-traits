#![feature(
)]

pub use ::enum_traits_proc_macros::*;
use ::num_traits::AsPrimitive;

pub unsafe trait DiscriminantValues {
	type Discriminant: 'static + Clone + Copy + std::fmt::Debug + Eq
		+ PartialEq<Self::Discriminant> + std::hash::Hash + Send + Sync + Unpin
		+ AsPrimitive<usize>
	;
	
	const VALUES: &'static [Self::Discriminant];

	const EVER_ENABLED_BITS: Self::Discriminant;
	const ALWAYS_ENABLED_BITS: Self::Discriminant;
	const MAX: Option<Self::Discriminant>;
	const COUNT: usize;
}

pub unsafe trait DiscriminantHeaded: DiscriminantValues {
	fn discriminant(&self) -> &Self::Discriminant {
		unsafe { &*(self as *const _ as *const _) }
	}

	fn unchanged_discriminant(&self) -> &UnchangedDiscriminant<Self> {
		unsafe { &*(self as *const _ as *const _) }
	}
}

pub unsafe trait ContinuousDiscriminants: DiscriminantValues {}

pub unsafe trait FirstDiscriminantIsZero: DiscriminantValues {}

#[derive(PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
#[repr(transparent)]
pub struct UnchangedDiscriminant<T: ?Sized + DiscriminantValues>(T::Discriminant);

impl<T> std::ops::Deref for UnchangedDiscriminant<T> where
	T: ?Sized + DiscriminantValues,
{
	type Target = T::Discriminant;
	fn deref(&self) -> &Self::Target { &self.0 }
}

pub trait DiscriminantIndex: FirstDiscriminantIsZero + ContinuousDiscriminants {
	fn discriminant_as_usize(v: Self::Discriminant) -> usize { v.as_() }
}

impl<T> UnchangedDiscriminant<T> where
	T: ?Sized + DiscriminantIndex,
{
	//TODO change to T::COUNT when this works; in the meantime, hope the assert is
	// optimized out

	pub fn index<U, const COUNT: usize>(self, array: &[U; COUNT]) -> &U {
		assert_eq!(COUNT, T::COUNT);
		unsafe { array.get_unchecked(T::discriminant_as_usize(self.0)) }
	}

	pub fn index_mut<U, const COUNT: usize>(self, array: &mut [U; COUNT]) -> &mut U {
		assert_eq!(COUNT, T::COUNT);
		unsafe { array.get_unchecked_mut(T::discriminant_as_usize(self.0)) }
	}
}

