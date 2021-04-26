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

			let output = f(&syn_item, &syn_enum, &reprs)?;

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

#[proc_macro_derive(DiscriminantValues)]
pub fn derive_discriminant_values(input: TokenStream) -> TokenStream {
	gen_enum_trait(|syn_item, syn_enum, reprs| {
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
	
		let mut last_discr = BigInt::default();

		let mut discrs = Vec::with_capacity(syn_enum.variants.len());

		for variant in syn_enum.variants.iter() {
			let value: BigInt = match &variant.discriminant {
				Some((_, syn::Expr::Lit(lit))) => match &lit.lit {
					syn::Lit::Int(lit) => lit.base10_digits().parse().unwrap(),
					other => panic!("enum discriminant must be Int, not `{:?}`", other),
				},
				None => last_discr + 1,
				_ => panic!("literal value must be a specified literal or unspecified"),
			};

			discrs.push(syn::LitInt::new(&format!("{}", &value), Span::call_site()));

			last_discr = value;
		}

		Ok(quote!(
			type Discriminant = ::std::primitive::#repr;
			const VALUES: &'static [Self::Discriminant] = &[#(#discrs),*];
		))
	})(quote!(DiscriminantValues), input)
}

#[proc_macro_derive(ReprC)]
pub fn derive_repr_c(input: TokenStream) -> TokenStream {
	gen_enum_trait(|syn_item, _syn_enum, reprs| {
		if !reprs.iter().flat_map(|v| v).any(|v| v == "C") {
			return Err(Error::new(syn_item.span(), "expected repr(C)"));
		}

		Ok(quote!())
	})(quote!(ReprC), input)
}

