#include <stdbool.h>
#include "wasm.h"

typedef unsigned long size_t;
typedef int std_obj_t;

typedef enum {
	STD_NULL = 0,
	STD_INT = 1,
	STD_FLOAT = 2,
	STD_STRING = 3,
	STD_BOOL = 4,
	STD_ARRAY = 5,
	STD_OBJECT = 6,
	STD_DATE = 7,
} std_object_type_t;

WASM_IMPORT("std", "copy") std_obj_t copy(void);
WASM_IMPORT("std", "destroy") void destroy(std_obj_t obj);

WASM_IMPORT("std", "create_null") std_obj_t create_null(void);
WASM_IMPORT("std", "create_int") std_obj_t create_int(long int number);
WASM_IMPORT("std", "create_float") std_obj_t create_float(double number);
WASM_IMPORT("std", "create_string") std_obj_t create_string(char *string, size_t string_len);
WASM_IMPORT("std", "create_bool") std_obj_t create_bool(bool boolean);
WASM_IMPORT("std", "create_array") std_obj_t create_array(void);
WASM_IMPORT("std", "create_object") std_obj_t create_object(void);
WASM_IMPORT("std", "create_date") std_obj_t create_date(void);

WASM_IMPORT("std", "typeof") std_object_type_t std_typeof(std_obj_t obj);
WASM_IMPORT("std", "string_len") int string_len(std_obj_t obj);
WASM_IMPORT("std", "read_string") void read_string(std_obj_t obj, char *buffer, size_t buffer_size);
WASM_IMPORT("std", "read_int") int read_int(std_obj_t obj);
WASM_IMPORT("std", "read_float") double read_float(std_obj_t obj);
WASM_IMPORT("std", "read_bool") bool read_bool(std_obj_t obj);
WASM_IMPORT("std", "read_date") double read_date(std_obj_t obj);
WASM_IMPORT("std", "read_date_string") double read_date_string(std_obj_t obj, char *format, size_t format_len, char *local, size_t local_len, char *timezone, size_t timezone_len);

WASM_IMPORT("std", "object_len") int object_len(std_obj_t obj);
WASM_IMPORT("std", "object_get") std_obj_t object_get(std_obj_t obj, char *key, size_t key_len);
WASM_IMPORT("std", "object_set") void object_set(std_obj_t obj, char *key, size_t key_len, std_obj_t value);
WASM_IMPORT("std", "object_remove") void object_remove(std_obj_t obj, char *key, size_t key_len);
WASM_IMPORT("std", "object_keys") std_obj_t object_keys(std_obj_t obj);
WASM_IMPORT("std", "object_values") std_obj_t object_values(std_obj_t obj);

WASM_IMPORT("std", "array_len") int array_len(std_obj_t obj);
WASM_IMPORT("std", "array_get") std_obj_t array_get(std_obj_t obj, int pos);
WASM_IMPORT("std", "array_set") void array_set(std_obj_t obj, int pos, std_obj_t value);
WASM_IMPORT("std", "array_remove") void array_remove(std_obj_t obj, int pos);
WASM_IMPORT("std", "array_append") void array_append(std_obj_t obj, int object);
