use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, PathArguments, Type};

#[proc_macro_derive(BuildNew, attributes(new, set, set_some))]
pub fn build_new(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let generics = input.generics;

    let Data::Struct(ds) = input.data else {
        panic!("can only operate on structs");
    };

    let Fields::Named(named_fields) = ds.fields else {
        panic!("can only operated on structs with named fields")
    };

    let mut new_args = Vec::new();
    let mut new_fields = Vec::new();
    let mut setters = Vec::new();
    for field in &named_fields.named {
        let mut already_in_fields = false;

        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        for attr in &field.attrs {
            let syn::Meta::Path(ref path) = attr.meta else {
                continue;
            };
            let quoted = quote!(#path).to_string();

            match quoted.as_str() {
                "new" => {
                    new_args.push(quote!(#ident: #ty));
                    new_fields.push(quote!(#ident));
                    already_in_fields = true;
                }
                "set" => {
                    setters.push(quote! {
                        pub fn #ident(mut self, #ident: #ty) -> Self {
                            self.#ident = #ident;
                            self
                        }
                    });
                }
                "set_some" => {
                    let Type::Path(ref inner_path) = ty else {
                        panic!("some_set requires an `Option<T>`")
                    };
                    assert!(inner_path.qself.is_none());
                    assert_eq!(inner_path.path.segments.len(), 1);

                    let PathArguments::AngleBracketed(option_args) =
                        &inner_path.path.segments[0].arguments
                    else {
                        panic!("expected angle brackets around `Option`")
                    };
                    let inner_type = &option_args.args;

                    setters.push(quote! {
                        pub fn #ident(mut self, #ident: #inner_type) -> Self {
                            self.#ident = Some(#ident);
                            self
                        }
                    });
                }
                _ => {}
            }
        }

        if !already_in_fields {
            new_fields.push(quote!(#ident: Default::default()));
        }
    }

    let expanded = quote! {
        impl #generics #name #generics {
            pub fn new(#(#new_args ,)*) -> Self {
                Self {
                    #(#new_fields ,)*
                }
            }

            #(#setters)*
        }
    };

    TokenStream::from(expanded)
}
