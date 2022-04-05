#include "wasm.h"

typedef unsigned long size_t;

WASM_IMPORT("env", "print") void print(char *message, size_t message_len);
