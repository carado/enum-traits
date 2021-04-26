# `enum-traits`: traits for enums

at the moment, the only trait is `DiscriminantValues`, defined as:

```rust
pub unsafe trait DiscriminantValues {
	type Discriminant: 'static + Clone + Copy + std::fmt::Debug + Eq
		+ PartialEq<Self::Discriminant> + std::hash::Hash + Send + Sync + Unpin;
	
	const VALUES: &'static [Self::Discriminant];
}
```
which can be derived with `#[derive(enum_traits::DiscriminantValues)]` on any enum, even generic ones.

