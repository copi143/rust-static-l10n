use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, LitStr, Token, parse_macro_input};

struct LangConfig {
    name: String,
    fallback: Option<String>,
}

struct Config {
    path: String,
    base: String,
    langs: Vec<LangConfig>,
}

struct F16nInput {
    fmt: LitStr,
    args: Punctuated<Expr, Token![,]>,
}

struct WriteL10nInput {
    target: Expr,
    key: LitStr,
}

struct WriteF16nInput {
    target: Expr,
    fmt: LitStr,
    args: Punctuated<Expr, Token![,]>,
}

struct L10nAssertInput {
    cond: Expr,
    key: LitStr,
}

struct L10nAssertCmpInput {
    left: Expr,
    right: Expr,
    key: LitStr,
}

struct F16nAssertInput {
    cond: Expr,
    fmt: LitStr,
    args: Punctuated<Expr, Token![,]>,
}

struct F16nAssertCmpInput {
    left: Expr,
    right: Expr,
    fmt: LitStr,
    args: Punctuated<Expr, Token![,]>,
}

impl Parse for F16nInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fmt: LitStr = input.parse()?;
        let args = if input.is_empty() {
            Punctuated::new()
        } else {
            input.parse::<Token![,]>()?;
            Punctuated::parse_terminated(input)?
        };
        Ok(F16nInput { fmt, args })
    }
}

impl Parse for L10nAssertInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let cond: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let key: LitStr = input.parse()?;
        Ok(L10nAssertInput { cond, key })
    }
}

impl Parse for L10nAssertCmpInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let left: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let right: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let key: LitStr = input.parse()?;
        Ok(L10nAssertCmpInput { left, right, key })
    }
}

impl Parse for F16nAssertInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let cond: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let fmt: LitStr = input.parse()?;
        let args = if input.is_empty() {
            Punctuated::new()
        } else {
            input.parse::<Token![,]>()?;
            Punctuated::parse_terminated(input)?
        };
        Ok(F16nAssertInput { cond, fmt, args })
    }
}

impl Parse for F16nAssertCmpInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let left: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let right: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let fmt: LitStr = input.parse()?;
        let args = if input.is_empty() {
            Punctuated::new()
        } else {
            input.parse::<Token![,]>()?;
            Punctuated::parse_terminated(input)?
        };
        Ok(F16nAssertCmpInput {
            left,
            right,
            fmt,
            args,
        })
    }
}

impl Parse for WriteL10nInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let target: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let key: LitStr = input.parse()?;
        Ok(WriteL10nInput { target, key })
    }
}

impl Parse for WriteF16nInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let target: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let fmt: LitStr = input.parse()?;
        let args = if input.is_empty() {
            Punctuated::new()
        } else {
            input.parse::<Token![,]>()?;
            Punctuated::parse_terminated(input)?
        };
        Ok(WriteF16nInput { target, fmt, args })
    }
}

fn parse_metadata(cfg: &toml::Value) -> Option<Config> {
    let path = cfg.get("path")?.as_str()?.to_string();
    let base = cfg.get("base")?.as_str()?.to_string();

    let langs_value = cfg.get("langs")?;
    let mut langs = Vec::new();

    if let Some(array) = langs_value.as_array() {
        for lang in array {
            if let Some(name) = lang.as_str() {
                langs.push(LangConfig {
                    name: name.to_string(),
                    fallback: None,
                });
            } else if let Some(table) = lang.as_table() {
                let name = table.get("name")?.as_str()?.to_string();
                let fallback = table
                    .get("fallback")
                    .and_then(|f| f.as_str())
                    .map(|s| s.to_string());
                langs.push(LangConfig { name, fallback });
            }
        }
    }

    Some(Config { path, base, langs })
}

fn get_metadata() -> Config {
    let manifest_path = Path::new(&*MANIFEST_DIR).join("Cargo.toml");
    let manifest_content = fs::read_to_string(manifest_path).unwrap();
    let manifest = toml::from_str::<toml::Value>(&manifest_content).unwrap();

    let cfg = manifest
        .get("package")
        .and_then(|pkg| pkg.get("metadata"))
        .and_then(|meta| meta.get("static-l10n"))
        .expect("static-l10n metadata not found in Cargo.toml");

    parse_metadata(cfg).expect("Invalid static-l10n metadata format")
}

static MANIFEST_DIR: LazyLock<String> =
    LazyLock::new(|| std::env::var("CARGO_MANIFEST_DIR").unwrap());
static CONFIG: LazyLock<Config> = LazyLock::new(|| get_metadata());
static TRANSLATION_PATH: LazyLock<String> = LazyLock::new(|| {
    let base_path = Path::new(&*MANIFEST_DIR).join(&CONFIG.path);
    base_path.to_string_lossy().to_string()
});
static TRANSLATIONS: LazyLock<HashMap<(String, String), String>> =
    LazyLock::new(|| parse_translations());

fn read_translate_files() -> Vec<String> {
    let file_paths = list_translate_file_paths();
    let mut files = Vec::new();
    for path in file_paths {
        if let Ok(content) = fs::read_to_string(&path) {
            files.push(content);
        }
    }
    files
}

fn list_translate_file_paths() -> Vec<String> {
    let mut files = Vec::new();

    let base_path = Path::new(&*TRANSLATION_PATH).to_path_buf();
    if !base_path.exists() || !base_path.is_dir() {
        return files;
    }

    let mut stack = vec![base_path];
    while let Some(current_dir) = stack.pop() {
        if let Ok(entries) = fs::read_dir(current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.is_file() {
                    files.push(path.to_string_lossy().to_string());
                }
            }
        }
    }

    files.sort();
    files
}

fn parse_translations() -> HashMap<(String, String), String> {
    let files = read_translate_files();
    let mut translations = HashMap::new();

    for content in files {
        if let Ok(toml_value) = toml::from_str::<toml::Value>(&content) {
            if let Some(table) = toml_value.as_table() {
                for (key, value) in table {
                    let _ = translations
                        .insert((key.clone(), CONFIG.base.clone()), key.clone())
                        .is_none_or(|_| {
                            panic!(
                                "Duplicate translation for key '{}' and lang '{}'",
                                key, CONFIG.base
                            )
                        });
                    if let Some(subtable) = value.as_table() {
                        for (lang, translation) in subtable {
                            if let Some(translation_str) = translation.as_str() {
                                let _ = translations
                                    .insert(
                                        (key.clone(), lang.clone()),
                                        translation_str.to_string(),
                                    )
                                    .is_none_or(|_| {
                                        panic!(
                                            "Duplicate translation for key '{}' and lang '{}'",
                                            key, lang
                                        )
                                    });
                            }
                        }
                    }
                }
            }
        }
    }

    translations
}

fn get_translations(key: &str) -> HashMap<String, Option<String>> {
    let mut result = HashMap::new();
    for lang in &CONFIG.langs {
        let mut fallback = lang.fallback.clone();
        let mut lang = lang.name.clone();
        let mut translation = TRANSLATIONS.get(&(key.to_string(), lang.clone())).cloned();
        while translation.is_none() {
            let Some(ref fb) = fallback else {
                break;
            };
            let fallbacked = CONFIG.langs.iter().find(|l| &l.name == fb).unwrap();
            fallback = fallbacked.fallback.clone();
            lang = fallbacked.name.clone();
            translation = TRANSLATIONS.get(&(key.to_string(), lang.clone())).cloned();
        }
        result.insert(lang.clone(), translation);
    }
    result
}

#[proc_macro]
pub fn debug_print_metadata(item: TokenStream) -> TokenStream {
    if !item.is_empty() {
        return quote! {
            compile_error!("debug_print_metadata does not take any arguments");
        }
        .into();
    }
    let span = proc_macro::Span::call_site();
    println!("Manifest dir: {}", *MANIFEST_DIR);
    println!("Debug info from file: {}", span.file());
    println!("static-l10n metadata:");
    println!("  path: {}", CONFIG.path);
    println!("  langs:");
    for lang in &CONFIG.langs {
        if let Some(fallback) = &lang.fallback {
            println!("    - name: {}, fallback: {}", lang.name, fallback);
        } else {
            println!("    - name: {}", lang.name);
        }
    }
    println!("Loaded translations:");
    for ((key, lang), translation) in &*TRANSLATIONS {
        println!(
            "  - key: {}, lang: {}, translation: {}",
            key, lang, translation
        );
    }
    quote! {}.into()
}

#[proc_macro]
pub fn main(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    if !item.is_empty() {
        return quote! {
            compile_error!("main does not take any arguments");
        }
        .into();
    }
    let include_paths = list_translate_file_paths();
    let include_files = include_paths
        .iter()
        .map(|path| LitStr::new(path, proc_macro2::Span::call_site()))
        .map(|path| {
            quote! {
                const _: &str = include_str!(#path);
            }
        });
    let default_lang = &CONFIG.base;
    quote! {
        #(#include_files)*
        static __STATIC_L10N_LANG__: ::std::sync::Mutex<&'static str> =
            ::std::sync::Mutex::new(#default_lang);
    }
    .into()
}

#[proc_macro]
pub fn lang(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let lang = syn::parse_macro_input!(item as syn::LitStr).value();
    quote! {
        {
            *crate::__STATIC_L10N_LANG__.lock().unwrap() = #lang;
        }
    }
    .into()
}

#[proc_macro]
pub fn l10n(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as LitStr);
    expand_l10n_expr(input).into()
}

// 格式化到 format 的数字翻译宏，支持 f16n!("xxx: {}", xxx)
#[proc_macro]
pub fn f16n(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as F16nInput);
    expand_f16n_expr(input).into()
}

#[proc_macro]
pub fn l10n_args(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as LitStr);
    expand_l10n_args_expr(input).into()
}

#[proc_macro]
pub fn f16n_args(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as F16nInput);
    expand_f16n_args_expr(input).into()
}

macro_rules! impl_print_macros {
    ($l10n:ident, $f16n:ident, [ $($mac:tt)* ]) => {
        #[proc_macro]
        pub fn $l10n(item: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
            let input = parse_macro_input!(item as LitStr);
            expand_l10n_print_stmt(input, quote!( $($mac)* )).into()
        }
        #[proc_macro]
        pub fn $f16n(item: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
            let input = parse_macro_input!(item as F16nInput);
            expand_f16n_print_stmt(input, quote!( $($mac)* )).into()
        }
    };
}

impl_print_macros!(l10n_print,    f16n_print,    [::std::print!]);
impl_print_macros!(l10n_println,  f16n_println,  [::std::println!]);
impl_print_macros!(l10n_eprint,   f16n_eprint,   [::std::eprint!]);
impl_print_macros!(l10n_eprintln, f16n_eprintln, [::std::eprintln!]);

macro_rules! impl_write_macros {
    ($l10n:ident, $f16n:ident, [ $($mac:tt)* ]) => {
        #[proc_macro]
        pub fn $l10n(item: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
            let input = parse_macro_input!(item as WriteL10nInput);
            expand_l10n_write_stmt(input, quote!( $($mac)* )).into()
        }
        #[proc_macro]
        pub fn $f16n(item: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
            let input = parse_macro_input!(item as WriteF16nInput);
            expand_f16n_write_stmt(input, quote!( $($mac)* )).into()
        }
    };
}

impl_write_macros!(l10n_write,   f16n_write,   [::std::write!]);
impl_write_macros!(l10n_writeln, f16n_writeln, [::std::writeln!]);

#[proc_macro]
pub fn l10n_panic(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as LitStr);
    expand_l10n_panic_stmt(input).into()
}

#[proc_macro]
pub fn f16n_panic(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as F16nInput);
    expand_f16n_panic_stmt(input).into()
}

macro_rules! impl_l10n_assert_macros {
    ($name:ident / $dbg:ident, cond) => {
        #[proc_macro]
        pub fn $name(item: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
            let input = parse_macro_input!(item as L10nAssertInput);
            expand_l10n_assert_inner(input.cond, input.key, quote!(::std::assert!)).into()
        }
        #[proc_macro]
        pub fn $dbg(item: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
            let input = parse_macro_input!(item as L10nAssertInput);
            expand_l10n_assert_inner(input.cond, input.key, quote!(::std::debug_assert!)).into()
        }
    };
    ($name:ident / $dbg:ident, eq) => {
        #[proc_macro]
        pub fn $name(item: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
            let input = parse_macro_input!(item as L10nAssertCmpInput);
            expand_l10n_assert_cmp_inner(input.left, input.right, input.key, quote!(::std::assert_eq!)).into()
        }
        #[proc_macro]
        pub fn $dbg(item: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
            let input = parse_macro_input!(item as L10nAssertCmpInput);
            expand_l10n_assert_cmp_inner(input.left, input.right, input.key, quote!(::std::debug_assert_eq!)).into()
        }
    };
    ($name:ident / $dbg:ident, ne) => {
        #[proc_macro]
        pub fn $name(item: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
            let input = parse_macro_input!(item as L10nAssertCmpInput);
            expand_l10n_assert_cmp_inner(input.left, input.right, input.key, quote!(::std::assert_ne!)).into()
        }
        #[proc_macro]
        pub fn $dbg(item: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
            let input = parse_macro_input!(item as L10nAssertCmpInput);
            expand_l10n_assert_cmp_inner(input.left, input.right, input.key, quote!(::std::debug_assert_ne!)).into()
        }
    };
}

impl_l10n_assert_macros!(l10n_assert       / l10n_debug_assert,       cond);
impl_l10n_assert_macros!(l10n_assert_eq    / l10n_debug_assert_eq,    eq);
impl_l10n_assert_macros!(l10n_assert_ne    / l10n_debug_assert_ne,    ne);

macro_rules! impl_f16n_assert_macros {
    ($name:ident / $dbg:ident, cond) => {
        #[proc_macro]
        pub fn $name(item: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
            let input = parse_macro_input!(item as F16nAssertInput);
            expand_f16n_assert_inner(input.cond, input.fmt, input.args, quote!(::std::assert!)).into()
        }
        #[proc_macro]
        pub fn $dbg(item: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
            let input = parse_macro_input!(item as F16nAssertInput);
            expand_f16n_assert_inner(input.cond, input.fmt, input.args, quote!(::std::debug_assert!)).into()
        }
    };
    ($name:ident / $dbg:ident, eq) => {
        #[proc_macro]
        pub fn $name(item: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
            let input = parse_macro_input!(item as F16nAssertCmpInput);
            expand_f16n_assert_cmp_inner(input.left, input.right, input.fmt, input.args, quote!(::std::assert_eq!)).into()
        }
        #[proc_macro]
        pub fn $dbg(item: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
            let input = parse_macro_input!(item as F16nAssertCmpInput);
            expand_f16n_assert_cmp_inner(input.left, input.right, input.fmt, input.args, quote!(::std::debug_assert_eq!)).into()
        }
    };
    ($name:ident / $dbg:ident, ne) => {
        #[proc_macro]
        pub fn $name(item: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
            let input = parse_macro_input!(item as F16nAssertCmpInput);
            expand_f16n_assert_cmp_inner(input.left, input.right, input.fmt, input.args, quote!(::std::assert_ne!)).into()
        }
        #[proc_macro]
        pub fn $dbg(item: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
            let input = parse_macro_input!(item as F16nAssertCmpInput);
            expand_f16n_assert_cmp_inner(input.left, input.right, input.fmt, input.args, quote!(::std::debug_assert_ne!)).into()
        }
    };
}

impl_f16n_assert_macros!(f16n_assert       / f16n_debug_assert,       cond);
impl_f16n_assert_macros!(f16n_assert_eq    / f16n_debug_assert_eq,    eq);
impl_f16n_assert_macros!(f16n_assert_ne    / f16n_debug_assert_ne,    ne);

fn expand_l10n_expr(input: LitStr) -> proc_macro2::TokenStream {
    let key = input.value();
    let span = input.span();
    let translations = build_l10n_arms(&key, span, |lang, translation, span| {
        quote_spanned! {span=>
            #lang => #translation
        }
    });
    expand_match_expr(span, translations)
}

fn expand_l10n_args_expr(input: LitStr) -> proc_macro2::TokenStream {
    let key = input.value();
    let span = input.span();
    let translations = build_l10n_arms(&key, span, |lang, translation, span| {
        quote_spanned! {span=>
            #lang => ::std::format_args!(#translation)
        }
    });
    expand_match_expr(span, translations)
}

fn expand_l10n_print_stmt(
    input: LitStr,
    printer: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let key = input.value();
    let span = input.span();
    let translations = build_l10n_arms(&key, span, |lang, translation, span| {
        quote_spanned! {span=>
            #lang => { #printer(#translation); }
        }
    });
    expand_match_expr(span, translations)
}

fn expand_l10n_write_stmt(
    input: WriteL10nInput,
    writer: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let key = input.key;
    let target = input.target;
    let key_value = key.value();
    let span = key.span();
    let translations = build_l10n_arms(&key_value, span, |lang, translation, span| {
        quote_spanned! {span=>
            #lang => { let _ = #writer(#target, #translation); }
        }
    });
    expand_match_expr(span, translations)
}

fn expand_l10n_panic_stmt(input: LitStr) -> proc_macro2::TokenStream {
    let key = input.value();
    let span = input.span();
    let translations = build_l10n_arms(&key, span, |lang, translation, span| {
        quote_spanned! {span=>
            #lang => ::std::panic!(#translation)
        }
    });
    expand_match_expr(span, translations)
}

fn expand_l10n_assert_inner(
    cond: Expr,
    key: LitStr,
    assert_mac: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let span = key.span();
    let message = expand_l10n_expr(key);
    quote_spanned! {span=>
        #assert_mac(#cond, "{}", #message);
    }
}

fn expand_l10n_assert_cmp_inner(
    left: Expr,
    right: Expr,
    key: LitStr,
    assert_mac: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let span = key.span();
    let message = expand_l10n_expr(key);
    quote_spanned! {span=>
        #assert_mac(#left, #right, "{}", #message);
    }
}

fn expand_f16n_assert_inner(
    cond: Expr,
    fmt: LitStr,
    args: Punctuated<Expr, Token![,]>,
    assert_mac: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let span = fmt.span();
    let message = expand_f16n_args_expr(F16nInput { fmt, args });
    quote_spanned! {span=>
        #assert_mac(#cond, "{}", #message);
    }
}

fn expand_f16n_assert_cmp_inner(
    left: Expr,
    right: Expr,
    fmt: LitStr,
    args: Punctuated<Expr, Token![,]>,
    assert_mac: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let span = fmt.span();
    let message = expand_f16n_args_expr(F16nInput { fmt, args });
    quote_spanned! {span=>
        #assert_mac(#left, #right, "{}", #message);
    }
}

fn expand_f16n_expr(input: F16nInput) -> proc_macro2::TokenStream {
    let fmt_str_lit = input.fmt;
    let fmt_str = fmt_str_lit.value();
    let span = fmt_str_lit.span();
    let translations = build_f16n_arms(
        &fmt_str,
        span,
        &input.args,
        |lang, translation, span, args| {
            quote_spanned! {span=>
                #lang => format!(#translation, #(#args),*)
            }
        },
    );
    expand_match_expr(span, translations)
}

fn expand_f16n_args_expr(input: F16nInput) -> proc_macro2::TokenStream {
    let fmt_str_lit = input.fmt;
    let fmt_str = fmt_str_lit.value();
    let span = fmt_str_lit.span();
    let translations = build_f16n_arms(
        &fmt_str,
        span,
        &input.args,
        |lang, translation, span, args| {
            quote_spanned! {span=>
                #lang => ::std::format_args!(#translation, #(#args),*)
            }
        },
    );
    expand_match_expr(span, translations)
}

fn expand_f16n_print_stmt(
    input: F16nInput,
    printer: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let fmt_str_lit = input.fmt;
    let fmt_str = fmt_str_lit.value();
    let span = fmt_str_lit.span();
    let translations = build_f16n_arms(
        &fmt_str,
        span,
        &input.args,
        |lang, translation, span, args| {
            quote_spanned! {span=>
                #lang => { #printer(#translation, #(#args),*); }
            }
        },
    );
    expand_match_expr(span, translations)
}

fn expand_f16n_write_stmt(
    input: WriteF16nInput,
    writer: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let fmt_str_lit = input.fmt;
    let fmt_str = fmt_str_lit.value();
    let target = input.target;
    let span = fmt_str_lit.span();
    let translations = build_f16n_arms(
        &fmt_str,
        span,
        &input.args,
        |lang, translation, span, args| {
            quote_spanned! {span=>
                #lang => { let _ = #writer(#target, #translation, #(#args),*); }
            }
        },
    );
    expand_match_expr(span, translations)
}

fn expand_f16n_panic_stmt(input: F16nInput) -> proc_macro2::TokenStream {
    let fmt_str_lit = input.fmt;
    let fmt_str = fmt_str_lit.value();
    let span = fmt_str_lit.span();
    let translations = build_f16n_arms(
        &fmt_str,
        span,
        &input.args,
        |lang, translation, span, args| {
            quote_spanned! {span=>
                #lang => ::std::panic!(#translation, #(#args),*)
            }
        },
    );
    expand_match_expr(span, translations)
}

fn expand_match_expr(
    span: proc_macro2::Span,
    translations: Vec<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    quote_spanned! {span=>
        match { crate::__STATIC_L10N_LANG__.lock().unwrap().clone() } {
            #(#translations,)*
            other => panic!("Unsupported language: {}", other),
        }
    }
}

fn build_l10n_arms(
    key: &str,
    span: proc_macro2::Span,
    mut make_arm: impl FnMut(&str, &str, proc_macro2::Span) -> proc_macro2::TokenStream,
) -> Vec<proc_macro2::TokenStream> {
    let translations = get_translations(key);
    let mut arms = Vec::with_capacity(translations.len());
    for (lang, translation) in translations {
        if let Some(translation) = translation {
            arms.push(make_arm(&lang, &translation, span));
        } else {
            let error_msg = format!("Missing translation for key '{}' in lang '{}'", key, lang);
            arms.push(quote_spanned! {span=>
                #lang => compile_error!(#error_msg)
            });
        }
    }
    arms
}

fn build_f16n_arms(
    fmt_str: &str,
    span: proc_macro2::Span,
    args: &Punctuated<Expr, Token![,]>,
    mut make_arm: impl FnMut(&str, &str, proc_macro2::Span, &[&Expr]) -> proc_macro2::TokenStream,
) -> Vec<proc_macro2::TokenStream> {
    let translations = get_translations(fmt_str);
    let args_vec: Vec<_> = args.iter().collect();
    let mut arms = Vec::with_capacity(translations.len());
    for (lang, translation) in translations {
        if let Some(translation) = translation {
            arms.push(make_arm(&lang, &translation, span, &args_vec));
        } else {
            let error_msg = format!(
                "Missing translation for key '{}' in lang '{}'",
                fmt_str, lang
            );
            arms.push(quote_spanned! {span=>
                #lang => compile_error!(#error_msg)
            });
        }
    }
    arms
}
