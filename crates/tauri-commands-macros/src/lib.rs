use heck::ToLowerCamelCase;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Lit, LitInt, LitStr, Visibility};

extern crate proc_macro;

#[proc_macro_attribute]
pub fn command(
    _att: proc_macro::TokenStream,
    ts: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut item_fn = parse_macro_input!(ts as ItemFn);

    let vis = item_fn.vis;
    let name = &item_fn.sig.ident;
    let name_str = LitStr::new(&name.to_string(), name.span());

    item_fn.vis = Visibility::Inherited;

    let mut doc_content = String::new();

    item_fn.attrs.retain(|attr| {
        if attr.path.is_ident("doc") {
            if let syn::Meta::NameValue(nv) = attr.parse_meta().unwrap() {
                if let Lit::Str(s) = nv.lit {
                    doc_content += &s.value();
                    doc_content += "\n";
                    return false;
                }
            }
        }

        true
    });

    for attr in &item_fn.attrs {
        if attr.path.is_ident("doc") {
            if let syn::Meta::NameValue(nv) = attr.parse_meta().unwrap() {
                if let Lit::Str(s) = nv.lit {
                    doc_content += &s.value();
                    doc_content += "\n";
                }
            }
        }
    }

    #[cfg(feature = "codegen")]
    let doc_str = LitStr::new(&doc_content, Span::call_site());

    #[cfg(feature = "codegen")]
    let arg_names = item_fn
        .sig
        .inputs
        .iter()
        .enumerate()
        .map(|(idx, n)| {
            let idx = idx + 1;
            match n {
                syn::FnArg::Typed(ty) => match &*ty.pat {
                    syn::Pat::Ident(id) => {
                        LitStr::new(&id.ident.to_string().to_lower_camel_case(), id.ident.span())
                    }
                    _ => LitStr::new(&format!("_{idx}"), Span::call_site()),
                },
                _ => LitStr::new(&format!("_{idx}"), Span::call_site()),
            }
        })
        .enumerate()
        .fold(quote! {}, |mut ts, (idx, arg_name)| {
            let idx = LitInt::new(&idx.to_string(), Span::call_site());
            ts.extend(quote! {
                __cmd.meta.args[#idx].name = #arg_name.into();
            });
            ts
        });

    #[cfg(feature = "codegen")]
    let codegen = quote! {
        __cmd.meta.docs = #doc_str.into();
        #arg_names
    };

    #[cfg(not(feature = "codegen"))]
    let codegen = quote! {};

    quote! {
        #[allow(non_camel_case_types)]
        #vis struct #name;

        impl tauri_commands::IntoCommand for #name {
            fn into_command<R: tauri::Runtime>(
                self,
                __commands: &mut tauri_commands::Commands<R>,
            ) -> (std::borrow::Cow<'static, str>, tauri_commands::Command<R>) {
                #item_fn
                let mut __cmd = __commands.create_command(#name);
                #codegen
                (#name_str.into(), __cmd)
            }
        }
    }
    .into()
}
