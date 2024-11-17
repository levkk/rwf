use rwf::http::Handler;
use rwf::prelude::*;

<% for route in routes %>use crate::controllers::<%= route %>::<%= route.camelize %>;
<% end %>
pub fn routes() -> Vec<Handler> {
    vec![<% for route in routes %>
        route!("/<%= route %>" => <%= route.camelize %>),<% end %>
    ]
}
