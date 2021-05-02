//use ::utl::*;

use {
	::proc_macro::TokenStream,
	::proc_macro2::{Span, TokenStream as TokenStream2},
	::quote::quote,
	::num_bigint::BigInt,
	::syn::{Result, Error, Ident, spanned::Spanned},
};

fn reprs(item: &syn::DeriveInput) -> Result<Vec<Vec<Ident>>> {
	struct Repr {
		_parens: syn::token::Paren,
		idents: syn::punctuated::Punctuated<syn::Ident, syn::Token![,]>,
	}

	impl syn::parse::Parse for Repr {
		fn parse(i: syn::parse::ParseStream) -> Result<Self> {
			let content;
			Ok(Repr {
				_parens: syn::parenthesized!(content in i),
				idents: content.parse_terminated(syn::Ident::parse)?,
			})
		}
	}

	let mut reprs = Vec::new();

	for attr in &item.attrs {
		if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "repr" {
			reprs.push(
				syn::parse2::<Repr>(attr.tokens.clone())?.idents.into_iter().collect()
			);
		}
	}

	Ok(reprs)
}

fn gen_enum_trait(
	f: impl FnOnce(
		&syn::DeriveInput,
		&syn::DataEnum,
		&Vec<Vec<Ident>>,
		&Vec<BigInt>,
	) -> Result<TokenStream2>,
) ->
	impl FnOnce(TokenStream2, TokenStream) -> TokenStream
{
	move |trait_name, input| {
		match (move || {
			let syn_item: syn::DeriveInput = syn::parse(input)?;

			let syn_enum = match &syn_item.data {
				syn::Data::Enum(syn_enum) => syn_enum,
				_ => return Err(Error::new(syn_item.span(), "expected enum")),
			};

			let reprs = reprs(&syn_item)?;

			let mut discrs = Vec::new();
			let mut next_discr = BigInt::default();

			for variant in syn_enum.variants.iter() {
				let value: BigInt = match &variant.discriminant {
					Some((_, syn::Expr::Lit(lit))) => match &lit.lit {
						syn::Lit::Int(lit) => lit.base10_digits().parse().unwrap(),
						other => panic!("enum discriminant must be Int, not `{:?}`", other),
					},
					None => next_discr,
					_ => panic!("literal value must be a specified literal or unspecified"),
				};

				next_discr = (&value) + 1;

				discrs.push(value);
			}

			let output = f(&syn_item, &syn_enum, &reprs, &discrs)?;

			Ok((syn_item, output))
		})() {
			Ok((syn_item, output)) => {
				let (impl_generics, ty_generics, where_clause) =
					syn_item.generics.split_for_impl();
				
				let enum_name = &syn_item.ident;
				
				quote!(
					unsafe impl #impl_generics ::enum_traits::#trait_name for
						#enum_name #ty_generics #where_clause
					{
						#output
					}
				)
			},
			Err(err) => err.into_compile_error(),
		}.into()
	}
}

fn render_bigint(value: &BigInt) -> syn::LitInt {
	syn::LitInt::new(&format!("{}", &value), Span::call_site())
}

fn quote_bigint(value: &BigInt) -> TokenStream2 {
	let lit = render_bigint(value);
	quote!(#lit)
}

#[proc_macro_derive(DiscriminantValues)]
pub fn derive_discriminant_discrs(input: TokenStream) -> TokenStream {
	gen_enum_trait(|syn_item, _syn_enum, reprs, discrs| {
		let repr = reprs.iter()
			.flat_map(|v| v)
			.filter(|repr| {
				[
					"bool", "char", "f32", "f64", "i128", "i16", "i32", "i64", "i8",
					"isize", "str", "u128", "u16", "u32", "u64", "u8", "usize",
				].iter().any(|t| repr == t)
			})
			.next()
			.ok_or_else(|| Error::new(
				syn_item.span(),
				"enum must have a repr( ) attribute of a ::std::primitive type",
			))?;
	
		let ever_enabled_bits = render_bigint(&
			discrs.iter().fold(BigInt::default(), |a, b| a | b),
		);

		let always_enabled_bits = render_bigint(&discrs.iter().fold(
			Option::<BigInt>::None,
			|opt_a, b| Some(match opt_a { None => b.clone(), Some(a) => a & b.clone() }),
		).unwrap_or_else(BigInt::default));

		let max = discrs.iter().max().map(quote_bigint).map_or_else(
			|| quote!(::std::option::Option::None),
			|v| quote!(::std::option::Option::Some(#v)),
		);

		let min = discrs.iter().min().map(quote_bigint).map_or_else(
			|| quote!(::std::option::Option::None),
			|v| quote!(::std::option::Option::Some(#v)),
		);

		let count = discrs.len();

		let discrs_lits = discrs.iter().map(render_bigint);

		Ok(quote!(
			type Discriminant = ::std::primitive::#repr;

			const VALUES: &'static [Self::Discriminant] = &[#(#discrs_lits),*];

			const EVER_ENABLED_BITS: Self::Discriminant = #ever_enabled_bits;
			const ALWAYS_ENABLED_BITS: Self::Discriminant = #always_enabled_bits;
			const MIN: ::std::option::Option<Self::Discriminant> = #min;
			const MAX: ::std::option::Option<Self::Discriminant> = #max;
			const COUNT: usize = #count;
		))
	})(quote!(DiscriminantValues), input)
}

#[proc_macro_derive(DiscriminantHeaded)]
pub fn derive_discriminant_headed(input: TokenStream) -> TokenStream {
	gen_enum_trait(|syn_item, syn_enum, reprs, _| {
		if !(
			reprs.iter().flat_map(|v| v).any(|v| v == "C")
			|| syn_enum.variants.iter().all(|v| v.fields == syn::Fields::Unit)
		) {
			return Err(Error::new(syn_item.span(), "expected repr(C)"));
		}

		Ok(quote!())
	})(quote!(DiscriminantHeaded), input)
}

#[proc_macro_derive(ContinuousDiscriminants)]
pub fn derive_continuous_discriminants(input: TokenStream) -> TokenStream {
	gen_enum_trait(|syn_item, _syn_enum, _reprs, discrs| {
		if !discrs.windows(2).all(|ar| ar[1] == ar[0].clone() + 1) {
			return Err(Error::new(syn_item.span(), "discontinuous discriminants"));
		}

		Ok(quote!())
	})(quote!(ContinuousDiscriminants), input)
}

#[proc_macro_derive(FirstDiscriminantIsZero)]
pub fn derive_first_discriminant_is_zero(input: TokenStream) -> TokenStream {
	gen_enum_trait(|syn_item, _syn_enum, _reprs, discrs| {
		if discrs.first().map_or(false, |v| v != &BigInt::default()) {
			return Err(Error::new(
				syn_item.span(),
				"first discriminant must be 0, unspecified, or not exist",
			));
		}

		Ok(quote!())
	})(quote!(FirstDiscriminantIsZero), input)
}

#[proc_macro_derive(FieldlessEnum)]
pub fn derive_fieldless_enum(input: TokenStream) -> TokenStream {
	gen_enum_trait(|syn_item, syn_enum, _reprs, _discrs| {
		if syn_enum.variants.iter().any(|v| v.fields != syn::Fields::Unit) {
			return Err(Error::new(syn_item.span(), "enum must have no fields"));
		}

		Ok(quote!())
	})(quote!(FieldlessEnum), input)
}

