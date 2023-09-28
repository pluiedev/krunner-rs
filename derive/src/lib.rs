use darling::ast::Data;
use darling::{FromDeriveInput, FromVariant};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Generics, Ident, LitStr};

#[derive(Debug, FromVariant)]
#[darling(attributes(action))]
struct ActionField {
	ident: Ident,

	id: LitStr,
	title: LitStr,
	icon: LitStr,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(action), supports(enum_unit))]
struct Action {
	ident: Ident,
	data: Data<ActionField, ()>,
	generics: Generics,
}

#[proc_macro_derive(Action, attributes(action))]
pub fn derive_action(input: TokenStream) -> TokenStream {
	let (ident, data, generics) = match Action::from_derive_input(&syn::parse_macro_input!(input)) {
		Ok(Action {
			ident,
			data,
			generics,
		}) => (ident, data, generics),
		Err(e) => return e.write_errors().into(),
	};
	let variants = data.take_enum().unwrap();

	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let variant_ids = variants.iter().map(|v| &v.ident);
	let from_ids = variants.iter().map(|ActionField { id, ident, .. }| {
		quote! { #id => ::std::option::Option::Some(Self::#ident), }
	});
	let to_ids = variants.iter().map(|ActionField { id, ident, .. }| {
		quote! { Self::#ident => #id, }
	});
	let infos = variants.iter().map(
		|ActionField {
		     ident, title, icon, ..
		 }| {
			quote! {
				Self::#ident => ::krunner::ActionInfo {
					title: ::std::string::String::from(#title),
					icon: ::std::string::String::from(#icon),
				},
			}
		},
	);

	quote! {
		#[automatically_derived]
		impl #impl_generics ::krunner::Action for #ident #ty_generics #where_clause {
			fn all() -> &'static [Self] {
				&[#(Self::#variant_ids),*]
			}
			fn from_id(s: &str) -> ::std::option::Option<Self> {
				match s {
					#(#from_ids)*
					_ => ::std::option::Option::None,
				}
			}
			fn to_id(&self) -> ::std::string::String {
				<::std::string::String as ::std::convert::From<&str>>::from(match self {
					#(#to_ids)*
				})
			}
			fn info(&self) -> ::krunner::ActionInfo {
				match self {
					#(#infos)*
				}
			}
		}
	}
	.into()
}
