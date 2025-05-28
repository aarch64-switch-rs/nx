use std::str::FromStr;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Gt, Lt, Mut, PathSep},
    AngleBracketedGenericArguments, FnArg, GenericArgument, Path, PathSegment, ReturnType,
    TraitItem, TraitItemFn, Type, TypePath,
};

pub fn ipc_trait(_args: TokenStream, ipc_trait: TokenStream) -> syn::Result<TokenStream> {
    let input: syn::ItemTrait = syn::parse2::<syn::ItemTrait>(ipc_trait)?;

    let name = input.ident;
    let vis = input.vis;

    for item in input.items.iter() {
        if !matches!(item, TraitItem::Fn(_)) {
            return Err(stringify_error(
                item.span(),
                "Only function items are supported for ipc_trait derivations",
            ));
        }
    }

    let mut client_fns = vec![];
    let mut server_fns = vec![];
    let mut handle_request_matches = vec![];
    let mut item_iter = input.items.iter();
    while let Some(TraitItem::Fn(fn_item)) = item_iter.next() {
        if fn_item.default.is_some() {
            return Err(stringify_error(
                fn_item.span(),
                "No default implementations are supported for ipc_trait derivations",
            ));
        }

        let fn_name = fn_item.sig.ident.clone();
        let mut ipc_rid: Option<u32> = None;
        let mut version_req = None;
        let mut return_type_is_session = false;
        for attr in fn_item.attrs.iter() {
            if let syn::Attribute {
                meta:
                    syn::Meta::List(syn::MetaList {
                        path,
                        delimiter: _,
                        tokens,
                    }),
                ..
            } = attr
            {
                if path.is_ident("ipc_rid") {
                    ipc_rid = Some(syn::parse2::<syn::LitInt>(tokens.clone())?.base10_parse()?);
                } else if path.is_ident("version") {
                    version_req = Some(syn::parse2::<syn::Expr>(tokens.clone())?);
                } else {
                    return Err(stringify_error(fn_item.span(), "Only the `ipc_rid`, `version` and `return_session` attrs are supported on ipc trait functions"));
                }
            } else if let syn::Attribute {
                meta: syn::Meta::Path(p),
                ..
            } = attr
            {
                if p.is_ident("return_session") {
                    return_type_is_session = true;
                } else {
                    return Err(stringify_error(fn_item.span(), "Only the `ipc_rid`, `version` and `return_session` attrs are supported on ipc trait functions"));
                }
            } else {
                return Err(stringify_error(fn_item.span(), "Only the `ipc_rid`, `version` and `return_session` attrs are supported on ipc trait functions"));
            }
        }

        if ipc_rid.is_none() {
            return Err(stringify_error(
                fn_item.span(),
                "IPC member functions must have an assigned request id",
            ));
        }

        let ipc_rid = ipc_rid.unwrap();

        let version_req = version_req
            .unwrap_or(syn::parse2(quote! {::nx::version::VersionInterval::all()}).unwrap());

        // fix up the return types of the client functions to return nx::result::Result
        let mut client_fn = fn_item.clone();
        client_fn.attrs = vec![];

        let mut in_param_names = vec![];
        let mut in_param_types = vec![];
        let _: () = client_fn
            .sig
            .inputs
            .iter()
            .skip(1)
            .map(|fn_args| {
                let arg_pat = match fn_args {
                    FnArg::Typed(pat) => pat,
                    _ => panic!(
                        "We should only have non-receiver arguments after skipping the first arg."
                    ),
                };

                in_param_names.push(arg_pat.pat.clone());
                in_param_types.push(arg_pat.ty.clone());
            })
            .collect();

        let mut out_param_names = vec![];
        let mut out_param_types = vec![];
        match client_fn.sig.output.clone() {
            ReturnType::Default => {}
            ReturnType::Type(_, ty) => match *ty {
                Type::Tuple(tuple) => {
                    let types: Vec<Type> = (0..)
                        .map_while(|off| tuple.elems.get(off).cloned())
                        .collect();

                    let _: () = types
                        .into_iter()
                        .enumerate()
                        .map(|(off, t)| {
                            out_param_names.push(format_ident!("out_param_{}", off));
                            out_param_types.push(t);
                        })
                        .collect();
                }
                Type::Paren(ty_pat) => {
                    out_param_names.push(format_ident!("out_param_0"));
                    out_param_types.push((*ty_pat.elem).clone());
                }
                Type::Path(ty_path) => {
                    out_param_names.push(format_ident!("out_param_0"));
                    out_param_types.push(Type::Path(ty_path));
                }
                _ => {
                    return Err(stringify_error(
                        client_fn.sig.output.span(),
                        "Only tuple types, paren-wrapped types, or paths are supported for return types",
                    ));
                }
            },
        }

        client_fn.default = Some(syn::parse2(quote! {
                {
                    let mut ctx = ::nx::ipc::CommandContext::new_client(self.get_session().object_info);

                    let mut walker = ::nx::ipc::DataWalker::new(core::ptr::null_mut());

                    #(::nx::ipc::client::RequestCommandParameter::before_request_write(&#in_param_names, &mut walker, &mut ctx)?;)*

                    ctx.in_params.data_size = walker.get_offset() as u32;

                    match self.get_session().object_info.protocol {
                        ::nx::ipc::CommandProtocol::Cmif => ::nx::ipc::cmif::client::write_request_command_on_msg_buffer(&mut ctx, Some(#ipc_rid), ::nx::ipc::cmif::DomainCommandType::SendMessage),
                        ::nx::ipc::CommandProtocol::Tipc => ::nx::ipc::tipc::client::write_request_command_on_msg_buffer(&mut ctx, #ipc_rid)
                    };

                    walker.reset_with(ctx.in_params.data_offset);
                    #( ::nx::ipc::client::RequestCommandParameter::before_send_sync_request(&#in_param_names, &mut walker, &mut ctx)?; )*

                    ::nx::svc::send_sync_request(self.get_session().object_info.handle)?;

                    match self.get_session().object_info.protocol {
                        ::nx::ipc::CommandProtocol::Cmif => ::nx::ipc::cmif::client::read_request_command_response_from_msg_buffer(&mut ctx)?,
                        ::nx::ipc::CommandProtocol::Tipc => ::nx::ipc::tipc::client::read_request_command_response_from_msg_buffer(&mut ctx)?
                    };

                    walker.reset_with(ctx.out_params.data_offset);
                    #( let #out_param_names = <#out_param_types as ::nx::ipc::client::ResponseCommandParameter<_>>::after_response_read(&mut walker, &mut ctx)?; )*

                    Ok(( #(#out_param_names as _),* ))
                }
        })?);
        client_fn.semi_token = None;

        let output_span = client_fn.sig.output.span();
        match &mut client_fn.sig.output {
            ReturnType::Default => {
                client_fn.sig.output = syn::parse2::<ReturnType>(
                    FromStr::from_str(" -> ::nx::result::Result<()>").unwrap(),
                )
                .unwrap()
            }
            ReturnType::Type(_, ty) => {
                let mut wrapped_result_generic = Punctuated::new();
                wrapped_result_generic.push(GenericArgument::Type(*ty.clone()));

                let mut result_path = Punctuated::new();
                result_path.push(PathSegment {
                    ident: Ident::new_raw("nx", output_span),
                    arguments: syn::PathArguments::None,
                });
                result_path.push(PathSegment {
                    ident: Ident::new_raw("result", output_span),
                    arguments: syn::PathArguments::None,
                });
                result_path.push(PathSegment {
                    ident: Ident::new_raw("Result", output_span),
                    arguments: syn::PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        colon2_token: None,
                        lt_token: Lt::default(),
                        args: wrapped_result_generic,
                        gt_token: Gt::default(),
                    }),
                });

                *ty = Box::new(Type::Path(TypePath {
                    qself: None,
                    path: Path {
                        leading_colon: Some(PathSep::default()),
                        segments: result_path,
                    },
                }));
            }
        };

        client_fns.push(client_fn);

        // fix the return type for the server function types if
        let mut server_fn = fn_item.clone();
        server_fn.attrs = vec![];
        
        if let Some(FnArg::Receiver(r)) = server_fn.sig.inputs.iter_mut().next() {
            // all server functions are considered &mut borrowing
            r.mutability = Some(Mut::default());
            r.ty = Box::new(syn::parse2(quote! {&mut Self}).unwrap());

        } else {
            return Err(stringify_error(server_fn.span(), "IPC traits with associated functions is not supported."));
        }

        if return_type_is_session {
            if let ReturnType::Type(_, bty) = server_fn.sig.output.clone()
                && let Type::Path(ty) = *bty
            {
                if ty.path.segments.len() != 1 {
                    return Err(stringify_error(server_fn.sig.output.span(), "Output type be a raw type name (the base name of the traits) the return type is marked as a session type"));
                }
                let out_type_ident = format!(
                    " -> impl I{}Server + 'static",
                    ty.path.segments.last().unwrap().ident
                );
                server_fn.sig.output =
                    syn::parse2::<ReturnType>(FromStr::from_str(out_type_ident.as_str())?)?;
            } else {
                return Err(stringify_error(server_fn.sig.output.span(), "Output type be a raw type name (the base name of the traits) the return type is marked as a session type"));
            }
        }

        // now that the return type for server functions has been fixed, we can apply the same T -> Result<T> from the client functions
        match &mut server_fn.sig.output {
            ReturnType::Default => {
                server_fn.sig.output = syn::parse2::<ReturnType>(
                    FromStr::from_str(" -> ::nx::result::Result<()>").unwrap(),
                )
                .unwrap()
            }
            ReturnType::Type(_, ty) => {
                let mut wrapped_result_generic = Punctuated::new();
                wrapped_result_generic.push(GenericArgument::Type(*ty.clone()));

                let mut result_path = Punctuated::new();
                result_path.push(PathSegment {
                    ident: Ident::new_raw("nx", Span::call_site()),
                    arguments: syn::PathArguments::None,
                });
                result_path.push(PathSegment {
                    ident: Ident::new_raw("result", Span::call_site()),
                    arguments: syn::PathArguments::None,
                });
                result_path.push(PathSegment {
                    ident: Ident::new_raw("Result", Span::call_site()),
                    arguments: syn::PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        colon2_token: None,
                        lt_token: Lt::default(),
                        args: wrapped_result_generic,
                        gt_token: Gt::default(),
                    }),
                });

                *ty = Box::new(Type::Path(TypePath {
                    qself: None,
                    path: Path {
                        leading_colon: Some(PathSep::default()),
                        segments: result_path,
                    },
                }));
            }
        };
        server_fns.push(server_fn);

        let server_impl_fn_name = format_ident!("sf_server_impl_{}", fn_name);
        let carry_state_names: Vec<Ident> = out_param_names
            .iter()
            .map(|ident| format_ident!("{}_carry_state", ident))
            .collect();
        let server_internal_fn: TraitItemFn = syn::parse2(quote! {
            #[allow(unused_assignments)]
            #[allow(unused_parens)]
            #[allow(unused_mut)]
            #[doc(hidden)]
            fn #server_impl_fn_name(&mut self, protocol: ::nx::ipc::CommandProtocol, mut ctx: &mut ::nx::ipc::server::ServerContext) -> ::nx::result::Result<()> {
                use ::nx::result::ResultBase;

                ctx.raw_data_walker = ::nx::ipc::DataWalker::new(ctx.ctx.in_params.data_offset);
                #( let #in_param_names = <#in_param_types as ::nx::ipc::server::RequestCommandParameter<_>>::after_request_read(&mut ctx)?; )*

                let ( #( #out_param_names ),* ) = self.#fn_name( #( #in_param_names ),* )?;

                ctx.raw_data_walker = ::nx::ipc::DataWalker::new(core::ptr::null_mut());
                #( let #carry_state_names = ::nx::ipc::server::ResponseCommandParameter::before_response_write(&#out_param_names, &mut ctx)?; )*
                ctx.ctx.out_params.data_size = ctx.raw_data_walker.get_offset() as u32;

                match protocol {
                    ::nx::ipc::CommandProtocol::Cmif => {
                        ::nx::ipc::cmif::server::write_request_command_response_on_msg_buffer(&mut ctx.ctx, ::nx::result::ResultSuccess::make(), ::nx::ipc::cmif::CommandType::Request);
                    },
                    ::nx::ipc::CommandProtocol::Tipc => {
                        ::nx::ipc::tipc::server::write_request_command_response_on_msg_buffer(&mut ctx.ctx, ::nx::result::ResultSuccess::make(), 16); // TODO: is this command type actually read/used/relevant?
                    }
                };

                ctx.raw_data_walker = ::nx::ipc::DataWalker::new(ctx.ctx.out_params.data_offset);
                #( ::nx::ipc::server::ResponseCommandParameter::after_response_write(#out_param_names, #carry_state_names, &mut ctx)?; )*

                Ok(())
            }
        })?;

        server_fns.push(server_internal_fn);
        handle_request_matches.push(quote! {
            #ipc_rid if (#version_req).contains(version) => {
                Some(self.#server_impl_fn_name(protocol, ctx))
            }
        });
    }

    let client_trait = format_ident!("I{}Client", name);
    let server_trait = format_ident!("I{}Server", name);
    Ok(quote! {
        #vis trait #client_trait: ::nx::ipc::client::IClientObject + Sync {
            #(
                #[allow(unused_parens)]
                #[allow(clippy::too_many_arguments)]
                #[allow(missing_docs)]
                #client_fns
            )*
        }

        #vis trait #server_trait: ::nx::ipc::server::ISessionObject + Sync {
            #(
                #[allow(unused_parens)]
                #[allow(clippy::too_many_arguments)]
                #[allow(missing_docs)]
                #server_fns
            )*

            /// The dynamic dispatch function that calls into the IPC server functions. This should only be called from the [`::nx::ipc::server::ServerManager`] and not from client code.
                /// Examples for implementing [`ISessionObject`][`::nx::ipc::server::ISessionObject`] or [`IMitmServerOject`][`::nx::ipc::server::IMitmServerObject`] can be found in the [`nx`] crate.
                fn try_handle_request_by_id(&mut self, req_id: u32, protocol: ::nx::ipc::CommandProtocol, ctx: &mut ::nx::ipc::server::ServerContext) -> Option<::nx::result::Result<()>> {
                    let version = ::nx::version::get_version();
                    match req_id {
                        #(
                            #handle_request_matches
                        ),*
                        _ => None
                    }
                }
        }
    })
}

fn stringify_error(span: proc_macro2::Span, msg: impl std::fmt::Display) -> syn::Error {
    syn::Error::new(span, msg)
}
