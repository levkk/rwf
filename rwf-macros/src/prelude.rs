use crate::Expr;
pub use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, ToTokens};
use std::collections::HashMap;
use std::path::PathBuf;

pub use quote::quote;
pub use syn::parse::*;
use syn::punctuated::Punctuated;
use syn::token::Bracket;
pub use syn::*;

pub struct Migrations {
    bootstap: PathBuf,
    migrate: PathBuf,
}

impl Parse for Migrations {
    fn parse(input: ParseStream) -> Result<Self> {
        let bootstap = input.parse::<LitStr>()?;
        eprintln!("{}", bootstap.value());
        input.parse::<Token![,]>()?;
        let migrate = input.parse::<LitStr>()?;
        let bootstap = PathBuf::from(bootstap.value());
        let migrate = PathBuf::from(migrate.value());
        Ok(Self { bootstap, migrate })
    }
}
#[derive(Debug)]
struct IncludeMigration {
    direction: Ident,
    id: i64,
    name: String,
    path: PathBuf,
}
fn parse_file_name(name: String, path: PathBuf) -> IncludeMigration {
    let mut i = name.split(".").take(2);
    let mut name_comp = i.next().unwrap().splitn(2, "_");
    let direction = format_ident!("{}", i.next().unwrap().to_string());
    let id = name_comp.next().unwrap().parse::<i64>().unwrap();
    let name = name_comp.next().unwrap().to_string();
    IncludeMigration {
        direction,
        id,
        name,
        path,
    }
}

pub fn build_migratiosn(input: Migrations) {
    let mut output = proc_macro2::TokenStream::new();
    ItemUse {
        attrs: vec![],
        vis: Visibility::Inherited,
        use_token: Default::default(),
        leading_colon: None,
        tree: UseTree::Path(UsePath {
            ident: Ident::new("super", Span::call_site()),
            colon2_token: Token![::](Span::call_site()),
            tree: Box::new(UseTree::Path(UsePath {
                ident: Ident::new("bootstrap", Span::call_site()),
                colon2_token: Token![::](Span::call_site()),
                tree: Box::new(UseTree::Name(UseName {
                    ident: Ident::new("RwfDatabaseSchema", Span::call_site()),
                })),
            })),
        }),
        semi_token: Default::default(),
    }
    .to_tokens(&mut output);
    let mut f = ItemFn {
        attrs: vec![],
        vis: Visibility::Public(Token![pub](Span::call_site())),
        sig: Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: Default::default(),
            ident: Ident::new("migrations", Span::call_site()),
            generics: Default::default(),
            paren_token: Default::default(),
            inputs: Default::default(),
            variadic: None,
            output: parse_quote! { -> Vec<RwfDatabaseSchema>},
        },
        block: Box::new(Block {
            brace_token: Default::default(),
            stmts: vec![],
        }),
    };
    let mut slice = ExprArray {
        attrs: vec![],
        bracket_token: Bracket::default(),
        elems: Punctuated::default(),
    };
    if input.migrate.is_file() {
        let mut files = input
            .bootstap
            .read_dir()
            .expect("Bootstrap dir is not readable")
            .map(|entry| entry.expect("Failed to read entry from dir"))
            .filter(|path| {
                path.file_type()
                    .expect("Failed t6o load file type from entry")
                    .is_file()
            })
            .map(|path| path.path())
            .collect::<Vec<_>>();
        let mut map: HashMap<i64, HashMap<Ident, IncludeMigration>> = HashMap::new();
        for file in files.into_iter() {
            let mig = parse_file_name(
                file.file_name().unwrap().to_string_lossy().to_string(),
                file,
            );
            eprintln!("{:?}", mig);
            map.entry(mig.id.clone())
                .or_default()
                .entry(mig.direction.clone())
                .insert_entry(mig);
        }
        let mut idx = 1;
        loop {
            if !map.contains_key(&idx) {
                break;
            }
            let inner = map.remove(&idx).unwrap();
            let mut i = inner.into_values();
            let a = i.next().unwrap();
            let b = i.next().unwrap();
            eprintln!("{:?} {:?}", a, b);
            let adir = a.direction.clone();
            let bdir = b.direction.clone();
            let id = a.id.clone();
            let name = a.name.clone();
            let fn1 = format!(
                "bootstrap/{}",
                a.path.file_name().unwrap().to_string_lossy().to_string()
            );
            let fn2 = format!(
                "bootstrap/{}",
                b.path.file_name().unwrap().to_string_lossy().to_string()
            );
            let res = parse_quote! {RwfDatabaseSchema {id: #id, name: #name.to_string(), #adir: include_str!(#fn1).to_string(), #bdir: include_str!(#fn2).to_string() }};
            slice.elems.push(res);
            idx += 1;
        }
        let slice = Expr::Array(slice);
        let call = Expr::MethodCall(ExprMethodCall {
            attrs: vec![],
            receiver: Box::new(slice),
            dot_token: Default::default(),
            method: Ident::new("to_vec", Span::call_site()),
            turbofish: None,
            paren_token: Default::default(),
            args: Default::default(),
        });
        f.block.stmts.push(Stmt::Expr(call, None));
        f.to_tokens(&mut output)
    }
    let res = prettyplease::unparse(&syn::parse_file(output.to_string().as_str()).unwrap());
    std::fs::write(input.migrate, res).unwrap();
}
