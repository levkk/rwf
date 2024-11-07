/*
 * Wrap various Ruby C macros into actual functions.
*/
#include <assert.h>
#include <stdlib.h>
#include <ruby.h>
#include <stdio.h>
#include "bindings.h"
#include "ruby/internal/arithmetic/int.h"
#include "ruby/internal/core/rstring.h"
#include "ruby/internal/eval.h"
#include "ruby/internal/special_consts.h"
#include "ruby/internal/value_type.h"

static void rwf_print_error(void);

void rwf_init_ruby() {
    ruby_setup();
    ruby_init_loadpath();
    ruby_script("rwf_loader");
}

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

typedef enum RackBody {
    PROXY,
    FILES,
} RackBody;


void rwf_debug_value(VALUE v) {
    int state;
    VALUE kernel = rb_eval_string_protect("Kernel", &state);
    VALUE str = rb_obj_as_string(v);
    rb_funcall(kernel, rb_intern("puts"), 1, str);

    VALUE methods = rb_funcall(v, rb_intern("inspect"), 0);
    rb_funcall(kernel, rb_intern("puts"), 1, methods);
}

int rwf_responds_to(VALUE value, const char* name) {
    if (value == Qnil) {
        return 0;
    }

    VALUE name_s = rb_str_new_cstr(name);
    VALUE responds_to = rb_funcall(value, rb_intern("respond_to?"), 1, name_s);

    /* I feel dumb, but Qtrue doesn't work. */
    VALUE object_id = rb_funcall(responds_to, rb_intern("object_id"), 0);
    int is_true = NUM2INT(object_id);

    if (is_true == 20) {
        return 0;
    } else {
        return 1;
    }
}

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


RackResponse rwf_rack_response_new(VALUE value) {
    assert(TYPE(value) == T_ARRAY);
    assert(RARRAY_LEN(value) == 3);

    VALUE headers = rb_ary_entry(value, 1);
    assert(TYPE(headers) == T_HASH);

    RackResponse response;

    response.code = NUM2INT(rb_ary_entry(value, 0));
    response.num_headers = RHASH_SIZE(headers);

    VALUE header_keys = rb_funcall(headers, rb_intern("keys"), 0);
    response.headers = malloc(response.num_headers * sizeof(EnvKey));

    for(int i = 0; i < response.num_headers; i++) {
        VALUE header_key = rb_ary_entry(header_keys, i);
        VALUE header_value = rb_hash_fetch(headers, header_key);

        /* There is a MRI function for this, but I can't find it anymore */
        VALUE header_key_symbol_str = rb_funcall(header_key, rb_intern("to_s"), 0);

        char *header_key_str = StringValueCStr(header_key_symbol_str);
        char *header_value_str = StringValueCStr(header_value);

        EnvKey env_key;
        env_key.key = header_key_str;
        env_key.value = header_value_str;

        response.headers[i] = env_key;
    }

    VALUE body_entry = rb_ary_entry(value, 2);

    // It can be an array or it can be a Proxy object which duck-types
    // to array.
    VALUE body = rwf_get_body(body_entry, &response.is_file);

    response.body = StringValueCStr(body);
    response.value = value;

    return response;
}

void rwf_debug_key(EnvKey *k) {
    int state;
    VALUE kernel = rb_eval_string_protect("Kernel", &state);

    VALUE key = rb_str_new_cstr(k->key);
    VALUE value = rb_str_new_cstr(k->value);

    rb_funcall(kernel, rb_intern("puts"), 1, key);
    rb_funcall(kernel, rb_intern("puts"), 1, value);
}

RackResponse rwf_app_call(RackRequest request) {
    int state;

    VALUE hash = rb_hash_new();
    for (int i = 0; i < request.length; i++) {
        VALUE key = rb_str_new_cstr(request.env[i].key);
        VALUE value = rb_str_new_cstr(request.env[i].value);

        rb_hash_aset(hash, key, value);
    }

    VALUE app = rb_eval_string_protect("Rails.application", &state);

    if (state) {
        rwf_print_error();
    }

    VALUE response = rb_funcall(app, rb_intern("call"), 1, hash);

    rwf_print_error();

    return rwf_rack_response_new(response);
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

void rwf_print_error() {
    VALUE error = rb_errinfo();

    if (error != Qnil) {
        VALUE error_str = rb_obj_as_string(error);
        char *error_msg = StringValueCStr(error_str);
        VALUE backtrace = rb_funcall(error, rb_intern("backtrace"), 0);
        VALUE backtrace_obj = rb_obj_as_string(backtrace);
        char *backtrace_str = StringValueCStr(backtrace_obj);
        printf("error: %s\nbacktrace: %s", error_msg, backtrace_str);
    }

    rb_set_errinfo(Qnil);
}
