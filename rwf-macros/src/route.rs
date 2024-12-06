use crate::prelude::*;

struct RouteInput {
    route: Expr,
    controller: Expr,
}

impl Parse for RouteInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let route = input.parse()?;
        let _arrow: Token![=>] = input.parse()?;

        let controller = input.parse()?;

        Ok(Self { route, controller })
    }
}

pub(crate) fn route_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as RouteInput);

    let route = input.route;
    let controller = input.controller;

    let controller = match controller {
        Expr::Path(expr) => quote! { #expr::default() },
        expr => quote! { #expr },
    };

    quote! {
        #controller.route(#route)
    }
    .into()
}
