#include "wasm.h"

typedef unsigned long size_t;
typedef int std_obj_t;

WASM_IMPORT("json", "parse") std_obj_t json_parse(unsigned char *data, size_t size);
