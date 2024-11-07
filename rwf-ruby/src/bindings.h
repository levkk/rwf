
#include <stdint.h>
#include <ruby.h>

#ifndef BINDINGS_H
#define BINDINGS_H

typedef struct EnvKey {
    const char *key;
    const char *value;
} KeyValue;

typedef struct RackResponse {
    uintptr_t value;
    int code;
    int num_headers;
    KeyValue *headers;
    char* body;
    int is_file;
} RackResponse;

typedef struct RackRequest {
    const KeyValue* env;
    const int length;
} RackRequest;


int rwf_load_app(const char *path);
void rwf_init_ruby(void);
RackResponse rwf_rack_response_new(VALUE value);
int rwf_app_call(RackRequest request, const char *app_name, RackResponse *res);

#endif
