use crate::prelude::*;

struct RenderInput {
    template_name: LitStr,
    _comma: Option<Token![,]>,
    context: Vec<ContextInput>,
    code: Option<LitInt>,
}

struct TurboStreamInput {
    template_name: LitStr,
    _comma_1: Token![,],
    id: Expr,
    _comma_2: Option<Token![,]>,
    context: Vec<ContextInput>,
}

impl TurboStreamInput {
    fn render_input(&self) -> RenderInput {
        RenderInput {
            template_name: self.template_name.clone(),
            _comma: self._comma_2.clone(),
            context: self.context.clone(),
            code: None,
        }
    }
}

impl Parse for TurboStreamInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let template_name: LitStr = input.parse()?;
        let _comma_1: Token![,] = input.parse()?;
        let id: Expr = input.parse()?;
        let _comma_2: Option<Token![,]> = input.parse()?;

        let mut context: Vec<ContextInput> = vec![];

        loop {
            match input.parse() {
                Ok(context_input) => context.push(context_input),
                Err(_) => break,
            };
        }

        Ok(TurboStreamInput {
            template_name,
            _comma_1,
            id,
            _comma_2,
            context,
        })
    }
}

#[derive(Clone)]
struct ContextInput {
    name: LitStr,
    _separator: Token![=>],
    value: Expr,
    _comma: Option<Token![,]>,
}

struct Context {
    values: Vec<ContextInput>,
}

impl Parse for Context {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut values = vec![];
        loop {
            let context: Result<ContextInput> = input.parse();

            if let Ok(context) = context {
                values.push(context);
            } else {
                break;
            }
        }

        Ok(Context { values })
    }
}

impl Parse for ContextInput {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ContextInput {
            name: input.parse()?,
            _separator: input.parse()?,
            value: input.parse()?,
            _comma: input.parse()?,
        })
    }
}

impl Parse for RenderInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let template_name: LitStr = input.parse()?;
        let _comma: Option<Token![,]> = input.parse()?;
        let mut code = None;

        let context = if _comma.is_some() {
            let mut result = vec![];
            loop {
                if input.peek(LitInt) {
                    let c: LitInt = input.parse().unwrap();
                    code = Some(c);
                } else {
                    let context: Result<ContextInput> = input.parse();

                    if let Ok(context) = context {
                        result.push(context);
                    } else {
                        break;
                    }
                }
            }

            result
        } else {
            vec![]
        };

        Ok(RenderInput {
            template_name,
            _comma,
            context,
            code,
        })
    }
}

fn render_template(input: &RenderInput) -> proc_macro2::TokenStream {
    let template_name = &input.template_name;

    let render_call = if input.context.is_empty() {
        vec![quote! {
            let html = template.render_default()?;
        }]
    } else {
        let mut values = vec![quote! {
            let mut context = rwf::view::template::Context::new();
        }];

        for value in &input.context {
            let name = &value.name;
            let val = &value.value;
            values.push(quote! {
                context.set(#name, #val)?;
            })
        }

        values.push(quote! {
            let html = template.render(&context)?;
        });

        values
    };

    quote! {
        let template = rwf::view::template::Template::load(#template_name)?;
        #(#render_call)*
    }
}

/// `render!` implementation.
pub fn render_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as RenderInput);
    let render_call = render_template(&input);

    let code = if let Some(code) = input.code {
        quote! {
            let response = response.code(#code);
        }
    } else {
        quote! {}
    };

    quote! {
        {
            #render_call

            let response = rwf::http::Response::new().html(html);
            #code
            return Ok(response)
        }
    }
    .into()
}

/// `context!` implementation.
pub fn context_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Context);
    let mut expansion = vec![quote! {
        let mut context = rwf::view::template::Context::new();
    }];

    for value in input.values {
        let name = value.name;
        let value = value.value;
        expansion.push(quote! {
            context.set(#name, #value)?;
        });
    }

    quote! {
        {
            #(#expansion)*
            context
        }
    }
    .into()
}

/// `turbo_stream!` implementation.
pub fn turbo_stream_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TurboStreamInput);
    let render_input = input.render_input();
    let render_call = render_template(&render_input);
    let id = input.id;

    quote! {
        {
            #render_call
            rwf::view::TurboStream::new(html).action("replace").target(#id)
        }
    }
    .into()
}
