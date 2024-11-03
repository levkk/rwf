<% for module in modules %>
pub use <%= module %>;
pub use <%= module %>::*;
<% end %>
