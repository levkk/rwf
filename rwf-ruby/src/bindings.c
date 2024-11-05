/*
 * Wrap various Ruby C macros into actual functions.
*/
#include <ruby.h>

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
