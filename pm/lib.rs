//use ::utl::*;

use {
	::proc_macro::TokenStream,
	::proc_macro2::Span,
	::quote::quote,
	::num_bigint::BigInt,
};

#[proc_macro_derive(DiscriminantValues)]
pub fn derive_enum_variant_count(input: TokenStream) -> TokenStream {
	let syn_item: syn::DeriveInput = syn::parse(input).unwrap();

	let syn_enum = match syn_item.data {
		syn::Data::Enum(syn_enum) => syn_enum,
		_ => panic!("BoundEnum only works on enums"),
	};

	let repr = syn_item.attrs.iter()
		.filter_map(|a| match &a.path {
			syn::Path { leading_colon: None, segments } => {
				let mut segs = segments.iter();
				let segm = match segs.next() { Some(v) => v, None => return None };
				if segs.next().is_some() { return None; }
				if segm.ident.to_string() != "repr" { return None; }

				dbg!(&a.tokens);

				let repr = match syn::parse2::<proc_macro2::TokenTree>(a.tokens.clone()) {
					Ok(proc_macro2::TokenTree::Group(group))
						if group.delimiter() == proc_macro2::Delimiter::Parenthesis
					=> match syn::parse2::<syn::Ident>(group.stream()) {
						Ok(ident) => ident,
						_ => return None,
					},
					_ => return None,
				};

				if ![
					"bool", "char", "f32", "f64", "i128", "i16", "i32", "i64", "i8",
					"isize", "str", "u128", "u16", "u32", "u64", "u8", "usize",
				].iter().any(|t| repr == t) {
					return None;
				}

				Some(repr)
			},
			_ => None,
		})
		.next()
		.unwrap_or_else(|| {
			panic!("enum must have a repr( ) attribute of a ::std::primitive type")
		});
	
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

	let (impl_generics, ty_generics, where_clause) =
		syn_item.generics.split_for_impl();
	
	let name = &syn_item.ident;
	
	quote!(
		unsafe impl #impl_generics ::enum_traits::DiscriminantValues for
			#name #ty_generics #where_clause
		{
			type Discriminant = ::std::primitive::#repr;

			const VALUES: &'static [Self::Discriminant] = &[#(#discrs),*];
		}
	).into()
}

