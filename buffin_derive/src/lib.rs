use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{DeriveInput, Expr, ExprLit, Lit, Meta, parse_macro_input, spanned::Spanned};

#[proc_macro_derive(ToBytes, attributes(tag))]
pub fn derive_to_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let mut type_tag_value = None;
    for attr in &input.attrs {
        if attr.path().is_ident("tag") {
            match &attr.meta {
                Meta::List(list) => {
                    if let Ok(expr) = list.parse_args::<Expr>() {
                        if let Expr::Lit(ExprLit {
                            lit: Lit::Str(lit_str),
                            ..
                        }) = expr
                        {
                            type_tag_value = Some(lit_str.value());
                        } else {
                            println!("invalid tag");
                        }
                    } else {
                        println!("invalid tag");
                    }
                }
                Meta::NameValue(nv) => {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }) = &nv.value
                    {
                        type_tag_value = Some(lit_str.value());
                    } else {
                        println!("invalid tag");
                    }
                }
                _ => {
                    println!("invalid tag");
                }
            }
        }
    }

    let add_type_tag = match &type_tag_value {
        Some(tag) => quote! {
            buffer.add_bytes(#tag.as_bytes())?;
        },
        None => quote! {},
    };

    match input.data {
        syn::Data::Struct(data_struct) => {
            let expanded = match data_struct.fields {
                syn::Fields::Named(fields_named) => {
                    let fields = fields_named.named.into_iter().map(|field| {
                        let field_name =
                            field
                                .ident
                                .clone()
                                .expect(&format!("{}:{}", file!(), line!()));
                        quote! {
                            buffer.add(&self.#field_name)?;
                        }
                    });

                    quote! {
                        impl buffin::ToBytes for #name {
                            fn to_bytes(&self, buffer: &mut [u8]) -> eyre::Result<usize> {
                                let mut buffer = Buffin::new(buffer);
                                #add_type_tag
                                #( #fields )*
                                Ok(buffer.len())
                            }
                        }
                    }
                }
                syn::Fields::Unnamed(fields_unnamed) => {
                    let field_bindings: Vec<_> = (0..fields_unnamed.unnamed.len())
                        .map(|i| syn::Ident::new(&format!("f{i}"), fields_unnamed.span()))
                        .collect();

                    let field_adds = field_bindings.iter().map(|binding| {
                        quote! {
                            buffer.add(#binding)?;
                        }
                    });

                    quote! {
                        impl buffin::ToBytes for #name {
                            fn to_bytes(&self, buffer: &mut [u8]) -> eyre::Result<usize> {
                                let mut buffer = Buffin::new(buffer);
                                #add_type_tag

                                let Self ( #( #field_bindings ),* ) = self;

                                #( #field_adds )*
                                Ok(buffer.len())
                            }
                        }
                    }
                }
                syn::Fields::Unit => {
                    if type_tag_value.is_none() {
                        panic!("unit structs must have a tag")
                    }
                    add_type_tag
                }
            };

            expanded.into()
        }
        syn::Data::Enum(data_enum) => {
            let variant_branches = data_enum.variants.iter().map(|variant| {
                let variant_ident = &variant.ident;
                let mut variant_name = variant_ident.to_string();

                for attr in &variant.attrs {
                    if attr.path().is_ident("tag") {
                        match &attr.meta {
                            Meta::List(list) => {
                                // handles #[tag("something")]
                                if let Ok(expr) = list.parse_args::<Expr>() {
                                    if let Expr::Lit(ExprLit {
                                        lit: Lit::Str(lit_str),
                                        ..
                                    }) = expr
                                    {
                                        variant_name = lit_str.value();
                                    }
                                }
                            }
                            Meta::NameValue(nv) => {
                                // handles #[tag = "something"]
                                if let Expr::Lit(ExprLit {
                                    lit: Lit::Str(lit_str),
                                    ..
                                }) = &nv.value
                                {
                                    variant_name = lit_str.value();
                                }
                            }
                            _ => {}
                        }
                    }
                }

                match &variant.fields {
                    syn::Fields::Unit => {
                        quote! {
                            Self::#variant_ident => {
                                buffer.add_bytes(#variant_name.as_bytes())?;
                            }
                        }
                    }
                    syn::Fields::Unnamed(fields_unnamed) => {
                        let field_bindings: Vec<_> = (0..fields_unnamed.unnamed.len())
                            .map(|i| syn::Ident::new(&format!("f{i}"), variant_ident.span()))
                            .collect();

                        let field_adds = field_bindings.iter().map(|binding| {
                            quote! {
                                buffer.add(#binding)?;
                            }
                        });

                        quote! {
                            Self::#variant_ident( #( #field_bindings ),* ) => {
                                buffer.add_bytes(#variant_name.as_bytes())?;
                                #( #field_adds )*
                            }
                        }
                    }
                    syn::Fields::Named(fields_named) => {
                        let field_idents: Vec<_> = fields_named
                            .named
                            .iter()
                            .map(|f| f.ident.as_ref().expect(&format!("{}:{}", file!(), line!())))
                            .collect();

                        let field_adds = field_idents.iter().map(|binding| {
                            quote! {
                                buffer.add(#binding)?;
                            }
                        });

                        quote! {
                            Self::#variant_ident { #( #field_idents ),* } => {
                                // TODO: This should be customizable by `#[tag("blah")]` on a variant.
                                buffer.add_bytes(#variant_name.as_bytes())?;
                                #( #field_adds )*
                            }
                        }
                    }
                }
            });

            let expanded = quote! {
                impl buffin::ToBytes for #name {
                    fn to_bytes(&self, buffer: &mut [u8]) -> eyre::Result<usize> {
                        let mut buffer = Buffin::new(buffer);
                        #add_type_tag
                        match &self {
                            #( #variant_branches )*
                        }
                        Ok(buffer.len())
                    }
                }
            };

            expanded.into()
        }
        other => {
            let other = format!("{other:?}");
            quote! {
                compile_error!("`#[derive(FromBytes)]` cannot be used for {:?}", #other);
            }
            .into()
        }
    }
}

#[proc_macro_derive(FromBytes)]
pub fn derive_from_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let mut type_tag_value = None;
    for attr in &input.attrs {
        if attr.path().is_ident("tag") {
            match &attr.meta {
                Meta::List(list) => {
                    if let Ok(expr) = list.parse_args::<Expr>() {
                        if let Expr::Lit(ExprLit {
                            lit: Lit::Str(lit_str),
                            ..
                        }) = expr
                        {
                            type_tag_value = Some(lit_str.value());
                        } else {
                            println!("invalid tag");
                        }
                    } else {
                        println!("invalid tag");
                    }
                }
                Meta::NameValue(nv) => {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }) = &nv.value
                    {
                        type_tag_value = Some(lit_str.value());
                    } else {
                        println!("invalid tag");
                    }
                }
                _ => {
                    println!("invalid tag");
                }
            }
        }
    }

    let get_type_tag = match &type_tag_value {
        Some(tag) => quote! {
            let (buffer, _) = nom::bytes::streaming::tag(#tag.as_bytes())(buffer)?;
        },
        None => quote! {},
    };

    match input.data {
        syn::Data::Struct(data_struct) => {
            let expanded = match data_struct.fields {
                syn::Fields::Named(fields_named) => {
                    let field_names = fields_named.named.iter().map(|field| {
                        field
                            .ident
                            .clone()
                            .expect(&format!("{}:{}", file!(), line!()))
                    });

                    let fields = fields_named.named.iter().map(|field| {
                        let field_name =
                            field
                                .ident
                                .as_ref()
                                .expect(&format!("{}:{}", file!(), line!()));
                        let ty = &field.ty;
                        let ty_tokens = ty.to_token_stream();

                        quote! {
                            let (buffer, #field_name) = <#ty_tokens>::from_bytes(buffer)?;
                        }
                    });

                    quote! {
                        impl buffin::FromBytes for #name {
                            fn from_bytes(buffer: &[u8]) -> nom::IResult<&[u8], Self> {
                                #get_type_tag
                                #( #fields )*
                                Ok(( buffer, Self { #( #field_names ),* }))
                            }
                        }
                    }
                }
                syn::Fields::Unnamed(fields_unnamed) => {
                    let field_bindings: Vec<_> = (0..fields_unnamed.unnamed.len())
                        .map(|i| syn::Ident::new(&format!("f{i}"), fields_unnamed.span()))
                        .collect();

                    let field_types: Vec<_> =
                        fields_unnamed.unnamed.iter().map(|f| &f.ty).collect();

                    let fields = field_types.into_iter().zip(&field_bindings).map(
                        |(field, field_binding)| {
                            let ty_tokens = field.to_token_stream();

                            // quote! { <#ty_tokens>::from_bytes }

                            quote! {
                                let (buffer, #field_binding) = <#ty_tokens>::from_bytes(buffer)?;
                            }
                        },
                    );

                    quote! {
                        impl buffin::FromBytes for #name {
                            fn from_bytes(buffer: &[u8]) -> nom::IResult<&[u8], Self> {
                                #get_type_tag
                                #( #fields )*
                                Ok(( buffer, Self ( #( #field_bindings ),* )))
                            }
                        }
                    }
                }
                syn::Fields::Unit => {
                    if type_tag_value.is_none() {
                        panic!("unit structs must have a tag")
                    }
                    get_type_tag
                }
            };

            expanded.into()
        }
        syn::Data::Enum(data_enum) => {
            let mut variant_tokens = TokenStream2::new();

            data_enum.variants.iter().for_each(|variant| {
                let variant_ident = &variant.ident;
                let mut variant_name = variant_ident.to_string();

                for attr in &variant.attrs {
                    if attr.path().is_ident("tag") {
                        match &attr.meta {
                            Meta::List(list) => {
                                // handles #[tag("something")]
                                if let Ok(expr) = list.parse_args::<Expr>() {
                                    if let Expr::Lit(ExprLit {
                                        lit: Lit::Str(lit_str),
                                        ..
                                    }) = expr
                                    {
                                        variant_name = lit_str.value();
                                    }
                                }
                            }
                            Meta::NameValue(nv) => {
                                // handles #[tag = "something"]
                                if let Expr::Lit(ExprLit {
                                    lit: Lit::Str(lit_str),
                                    ..
                                }) = &nv.value
                                {
                                    variant_name = lit_str.value();
                                }
                            }
                            _ => {}
                        }
                    }
                }

                match &variant.fields {
                    syn::Fields::Unit => {
                        variant_tokens.extend(quote! {
                            map(
                                tag(#variant_name),
                                |_| Self::#variant_ident,
                            ),
                        });
                    }
                    syn::Fields::Unnamed(fields_unnamed) => {
                        let field_bindings: Vec<_> = (0..fields_unnamed.unnamed.len())
                            .map(|i| syn::Ident::new(&format!("f{i}"), variant_ident.span()))
                            .collect();

                        let field_types: Vec<_> = fields_unnamed.unnamed.iter().map(|f| &f.ty).collect();

                        let fields = field_types.into_iter().map(|field| {
                            // HERE
                            let ty_tokens = field.to_token_stream();
                            quote! { <#ty_tokens>::from_bytes }
                        });

                        variant_tokens.extend(quote! {
                            map(
                                (tag(#variant_name), #( #fields ),*),
                                |(_, #( #field_bindings ),* )| Self::#variant_ident( #( #field_bindings ),* ),
                            ),
                        });

                    }
                    syn::Fields::Named(fields_named) => {
                        let field_idents: Vec<_> = fields_named
                            .named
                            .iter()
                            .map(|f| f.ident.as_ref().expect(&format!("{}:{}", file!(), line!())))
                            .collect();

                        let field_types: Vec<_> = fields_named.named.iter().map(|f| &f.ty).collect();

                        let fields = field_types.into_iter().map(|field| {
                            // HERE
                            let ty_tokens = field.to_token_stream();
                            quote! { <#ty_tokens>::from_bytes }
                        });

                        variant_tokens.extend(quote! {
                            map(
                                (tag(#variant_name), #( #fields ),*),
                                |(_, #( #field_idents ),* )| Self::#variant_ident{ #( #field_idents ),* },
                            ),
                        });
                    }
                }
            });

            let expanded = quote! {
                impl buffin::FromBytes for #name {
                    fn from_bytes(buffer: &[u8]) -> nom::IResult<&[u8], Self> {
                        use nom::{Parser, branch::alt, combinator::map, bytes::complete::tag};
                        #get_type_tag
                        alt(( #variant_tokens )).parse(buffer)
                    }
                }
            };

            expanded.into()
        }
        other => {
            let other = format!("{other:?}");
            quote! {
                compile_error!("`#[derive(FromBytes)]` cannot be used for {:?}", #other);
            }
            .into()
        }
    }
}
