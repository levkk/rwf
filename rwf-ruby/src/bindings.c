/*
 * Wrap various Ruby C macros into actual functions.
*/
#include <assert.h>
#include <stdlib.h>
#include <ruby.h>
#include <stdio.h>
#include "bindings.h"
#include "ruby/internal/intern/string.h"
#include "ruby/internal/special_consts.h"
#include "ruby/internal/symbol.h"


static int rwf_print_error(void);

static VALUE rwf_get_class(const char *name) {
    int state;
    VALUE clss = rb_eval_string_protect(name, &state);

    if (state == 0) {
        return clss;
    } else {
        rwf_print_error();
        return Qnil;
    }
}

void rwf_init_ruby() {
    ruby_setup();
    ruby_init_loadpath();
    ruby_script("rwf_loader");
}

/*
 * Load the Ruby app into memory.
 * This is the only known way to execute Ruby apps from C in a way that works.
*/
int rwf_load_app(const char* path) {
    int state;
    void *node;

    /* Ruby code to load the app. */
    char *require = malloc(strlen(path) + strlen("-erequire '") + strlen("'") + 1);
    sprintf(require, "-erequire '%s'", path);

    char* options[] = {
        "-v",
        require,
    };

    node = ruby_options(2, options);

    if (ruby_executable_node(node, &state)) {
        state = ruby_exec_node(node);

        if (state) {
            rwf_print_error();
            return -1;
        }
    } else {
        rwf_print_error();
        return -1;
    }

    free(require);
    return 0;
}

/*
 * Inspect a Ruby value and print that to the standard output.
*/
void rwf_debug_value(VALUE v) {
    int state;
    VALUE kernel = rb_eval_string_protect("Kernel", &state); /* Kernel is always available. */
    VALUE str = rb_obj_as_string(v);
    rb_funcall(kernel, rb_intern("puts"), 1, str);

    VALUE methods = rb_funcall(v, rb_intern("inspect"), 0);
    rb_funcall(kernel, rb_intern("puts"), 1, methods);
}

static int rwf_is_true(VALUE v) {
    /* I feel dumb, but Qtrue doesn't work. */
    VALUE object_id = rb_funcall(v, rb_intern("object_id"), 0);
    int is_true = NUM2INT(object_id);

    if (is_true == 20) {
        return 0;
    } else {
        return 1;
    }
}

static int rwf_is_nil(VALUE v) {
    return rwf_is_true(rb_funcall(v, rb_intern("nil?"), 0));
}

/*
 * Check if this value can accept a method.
 * Use this unless you're sure of the data type you're dealing with.
 * If you're wrong, the VM will segfault though.
*/
int rwf_responds_to(VALUE value, const char* name) {
    if (value == Qnil) {
        return 0;
    }

    VALUE name_s = rb_str_new_cstr(name);
    VALUE responds_to = rb_funcall(value, rb_intern("respond_to?"), 1, name_s);

    return rwf_is_true(responds_to);
}

/*
 * Try to figure out what Rack returned as the body.
 *
 * So far I've discovered it can either be a BodyProxy which duck-types to array,
 * or a File::Iterator which I'm not sure what it does, but I can get the path to the file,
 * which Rust can then read.
*/
VALUE rwf_get_body(VALUE value, int *is_file) {
    if (rwf_responds_to(value, "to_ary") == 0) {
        VALUE proxy_body_ar = rb_funcall(value, rb_intern("to_ary"), 0);
        *is_file = 0;
        return rb_ary_entry(proxy_body_ar, 0);
    } else if (rwf_responds_to(value, "path") == 0) {
        VALUE path = rb_funcall(value, rb_intern("path"), 0);
        *is_file = 1;
        return path;
    } else {
        *is_file = 0;
        return Qnil;
    }
}

/* Parse the response from Rack. */
RackResponse rwf_rack_response_new(VALUE value) {
    /*
        Rack returns an array of 3 elements:
          - HTTP code
          - headers hash
          - response body, which can be a few things
    */
    assert(TYPE(value) == T_ARRAY);
    assert(RARRAY_LEN(value) == 3);

    VALUE headers = rb_ary_entry(value, 1);
    assert(TYPE(headers) == T_HASH);

    RackResponse response;

    response.code = NUM2INT(rb_ary_entry(value, 0));
    response.num_headers = RHASH_SIZE(headers);

    VALUE header_keys = rb_funcall(headers, rb_intern("keys"), 0);
    response.headers = malloc(response.num_headers * sizeof(KeyValue));

    for(int i = 0; i < response.num_headers; i++) {
        VALUE header_key = rb_ary_entry(header_keys, i);
        VALUE header_value = rb_hash_fetch(headers, header_key);

        /* There is a MRI function for this, but I can't find it anymore */
        VALUE header_key_symbol_str = rb_funcall(header_key, rb_intern("to_s"), 0);

        char *header_key_str = StringValueCStr(header_key_symbol_str);
        char *header_value_str = StringValueCStr(header_value);

        KeyValue env_key;
        env_key.key = header_key_str;
        env_key.value = header_value_str;

        response.headers[i] = env_key;
    }

    VALUE body_entry = rb_ary_entry(value, 2);

    // It can be an array or it can be a Proxy object which duck-types
    // to array.
    VALUE body = rwf_get_body(body_entry, &response.is_file);

    if (rwf_is_nil(body) == 0) {
        VALUE empty = rb_str_new_cstr("");
        response.body = StringValueCStr(empty);
    } else {
        response.body = StringValueCStr(body);
    }

    response.value = value;

    return response;
}

static VALUE rwf_request_body(const char *body) {
    VALUE rb_str = rb_str_new_cstr(body);
    VALUE str_io = rwf_get_class("StringIO");
    VALUE wrapper = rwf_get_class("Rack::Lint::Wrapper::InputWrapper");

    VALUE str_io_instance = rb_funcall(str_io, rb_intern("new"), 1, rb_str);
    VALUE wrapper_instance = rb_funcall(wrapper, rb_intern("new"), 1, str_io_instance);

    return wrapper_instance;
}

/*
 * Execute a Rack app and return an HTTP response.
 *
 * The app_name is a Ruby string which evaluates to the Rack app, for example: `Rails.application`.
 *
 * This function isn't super safe yet. For example, if the app_name is not a Rack app, we'll segfault.
*/
int rwf_app_call(RackRequest request, const char *app_name, RackResponse *res) {
    int state;
    VALUE body = rwf_request_body(request.body);

    VALUE env = rb_hash_new();
    for (int i = 0; i < request.length; i++) {
        VALUE key = rb_str_new_cstr(request.env[i].key);
        VALUE value = rb_str_new_cstr(request.env[i].value);

        rb_hash_aset(env, key, value);
    }

    VALUE body_key = rb_str_new_cstr("rack.input");
    rb_hash_aset(env, body_key, body);

    VALUE app = rb_eval_string_protect(app_name, &state);

    if (state) {
        rwf_print_error();
        return -1;
    }

    VALUE response = rb_funcall(app, rb_intern("call"), 1, env);

    if (rwf_print_error() != 0) {
        return -1;
    }

    *res = rwf_rack_response_new(response);

    return 0;
}

void rwf_rack_response_drop(RackResponse *response) {
    free(response->headers);
}

/*
 * Get the Ruby value data type.
*/
int rwf_rb_type(VALUE value) {
    return (int)(rb_type(value));
}

/*
 * Convert the value into a C-string.
*/
char* rwf_value_cstr(VALUE value) {
    return StringValueCStr(value);
}

/*
 * Clear error state when an exception is thrown.
*/
void rwf_clear_error_state() {
    rb_set_errinfo(Qnil);
}

int rwf_print_error() {
    VALUE error = rb_errinfo();

    if (error != Qnil) {
        VALUE error_str = rb_obj_as_string(error);
        char *error_msg = StringValueCStr(error_str);
        VALUE backtrace = rb_funcall(error, rb_intern("backtrace"), 0);
        VALUE backtrace_obj = rb_obj_as_string(backtrace);
        char *backtrace_str = StringValueCStr(backtrace_obj);
        printf("error: %s\nbacktrace: %s", error_msg, backtrace_str);
        rb_set_errinfo(Qnil);
        return 1;
    }

    return 0;
}
