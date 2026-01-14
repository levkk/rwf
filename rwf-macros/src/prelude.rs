use crate::Expr;
pub use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use std::path::PathBuf;

pub use quote::quote;
pub use syn::parse::*;
use syn::punctuated::Punctuated;
use syn::token::{Brace, Bracket, Paren};
pub use syn::*;

pub struct MigrationsPath {
    prefix: PathBuf,
    bootstap: PathBuf,
    migrate: PathBuf,
}

impl Parse for MigrationsPath {
    fn parse(input: ParseStream) -> Result<Self> {
        let prefix = input.parse::<syn::LitStr>()?;
        input.parse::<Token![,]>()?;
        let bootstap = input.parse::<LitStr>()?;
        input.parse::<Token![,]>()?;
        let migrate = input.parse::<LitStr>()?;
        let prefix = PathBuf::from(prefix.value());
        let bootstap = PathBuf::from(bootstap.value());
        let migrate = PathBuf::from(migrate.value());
        Ok(Self {
            prefix,
            bootstap,
            migrate,
        })
    }
}

#[derive(Debug)]
struct IncludeMigration {
    id: i64,
    path: PathBuf,
}
fn parse_file_name(name: String, path: PathBuf) -> IncludeMigration {
    let mut i = name.split(".").take(2);
    let mut name_comp = i.next().unwrap().splitn(2, "_");
    let id = name_comp.next().unwrap().parse::<i64>().unwrap();
    IncludeMigration { id, path }
}

fn gen_imports(output: &mut proc_macro2::TokenStream) {
    let mut items = Punctuated::new();
    items.push(UseTree::Name(UseName {
        ident: Ident::new("RwfDatabaseSchema", Span::call_site()),
    }));
    items.push(UseTree::Name(UseName {
        ident: Ident::new("parse_database_schema", Span::call_site()),
    }));
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
                tree: Box::new(UseTree::Group(UseGroup {
                    brace_token: Brace::default(),
                    items,
                })),
            })),
        }),
        semi_token: Default::default(),
    }
    .to_tokens(output);
}
fn gen_migration_fn() -> ItemFn {
    ItemFn {
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
    }
}

fn latest_id(migrate_file: &File, f: &ItemFn) -> Option<i64> {
    for item in migrate_file.items.iter() {
        if let Item::Fn(item_fn) = item {
            if item_fn.sig.ident == f.sig.ident {
                for stmt in item_fn.block.stmts.iter() {
                    if let Stmt::Expr(Expr::MethodCall(meth), ..) = stmt {
                        if let Expr::Array(arr) = meth.receiver.as_ref() {
                            return Some(arr.elems.len() as i64);
                        }
                    }
                }
            }
        }
    }
    None
}

fn parse_new_migrations(input: &MigrationsPath, current_id: i64, slice: &mut ExprArray) {
    let bootstrap = input.prefix.join(input.bootstap.clone());
    let files = bootstrap
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

    for file in files.into_iter() {
        let mig = parse_file_name(
            file.file_name().unwrap().to_string_lossy().to_string(),
            file,
        );
        let mut parse_path = Punctuated::new();
        parse_path.push(PathSegment {
            ident: Ident::new("parse_database_schema", Span::call_site()),
            arguments: Default::default(),
        });
        let mut include_str_macro_path = Punctuated::new();
        include_str_macro_path.push(PathSegment {
            ident: Ident::new("include_str", Span::call_site()),
            arguments: Default::default(),
        });
        let include_str_macro_path = Path {
            leading_colon: None,
            segments: include_str_macro_path,
        };
        let parse_call = Expr::Path(ExprPath {
            attrs: vec![],
            qself: None,
            path: Path {
                leading_colon: None,
                segments: parse_path,
            },
        });
        //eprintln!("{} - {:?}", current_id, mig);
        if mig.id <= current_id {
            continue;
        } else {
            let parse_path_include = input
                .bootstap
                .join(mig.path.file_name().unwrap().to_string_lossy().to_string())
                .to_string_lossy()
                .to_string();
            let mut args = Punctuated::new();
            args.push(Expr::Macro(ExprMacro {
                attrs: vec![],
                mac: Macro {
                    path: include_str_macro_path.clone(),
                    bang_token: Default::default(),
                    delimiter: MacroDelimiter::Paren(Paren::default()),
                    tokens: parse_path_include.to_token_stream(),
                },
            }));
            slice.elems.push(Expr::Call(ExprCall {
                attrs: vec![],
                func: Box::new(parse_call.clone()),
                paren_token: Default::default(),
                args,
            }));
        }
    }
}

pub fn build_migratiosn(input: MigrationsPath) {
    let mut output = proc_macro2::TokenStream::new();
    let mut f = gen_migration_fn();
    let migrate = input.prefix.join(input.migrate.clone());
    if migrate.is_file() {
        let mut migrate_file =
            parse_file(std::fs::read_to_string(migrate.clone()).unwrap().as_str()).unwrap();
        let current_id = latest_id(&migrate_file, &f).unwrap_or(0);
        //eprintln!("{}", current_id);
        for item in migrate_file.items.iter_mut() {
            if let Item::Fn(item_fn) = item {
                if item_fn.sig.ident == f.sig.ident {
                    for stmt in item_fn.block.stmts.iter_mut() {
                        if let Stmt::Expr(Expr::MethodCall(ref mut meth), ..) = stmt {
                            if let Expr::Array(ref mut arr) = meth.receiver.as_mut() {
                                //eprintln!("{}", arr.to_token_stream());
                                parse_new_migrations(&input, current_id, arr);
                                //eprintln!("{}", arr.to_token_stream());
                            }
                        }
                    }
                }
            }
        }
        migrate_file.to_tokens(&mut output);
    } else {
        gen_imports(&mut output);

        let mut slice = ExprArray {
            attrs: vec![],
            bracket_token: Bracket::default(),
            elems: Punctuated::default(),
        };
        parse_new_migrations(&input, 0, &mut slice);
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
    if !output.is_empty() {
        let res = prettyplease::unparse(&parse_file(output.to_string().as_str()).unwrap());
        std::fs::write(migrate, res).unwrap();
    }
}
