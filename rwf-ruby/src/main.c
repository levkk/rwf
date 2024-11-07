#include "bindings.h"
#include <ruby.h>

int main() {
    int state;

    rwf_init_ruby();
    rwf_load_app("/home/lev/code/rwf/rwf-ruby/tests/todo/config/environment.rb");

    VALUE response = rb_eval_string_protect("Rails.application.call({})", &state);
    RackResponse res = rwf_rack_response_new(response);
}
