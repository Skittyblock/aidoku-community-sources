#define WASM_IMPORT(x, y) __attribute__((import_module(x))) __attribute__((import_name(y))) 
#define WASM_EXPORT __attribute__((visibility("default")))
#define WASM_EXPORT_AS __attribute__((export_name(x)))
