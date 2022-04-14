#include <stdbool.h>
#include "wasm.h"

typedef unsigned long size_t;
typedef int std_obj_t;

WASM_IMPORT("html", "parse") std_obj_t html_parse(unsigned char *data, size_t size);
WASM_IMPORT("html", "parse_fragment") std_obj_t html_parse_fragment(unsigned char *data, size_t size);

WASM_IMPORT("html", "select") std_obj_t html_select(std_obj_t obj, char *selector, size_t selector_len);
WASM_IMPORT("html", "attr") std_obj_t html_attr(std_obj_t obj, char *selector, size_t selector_len);

WASM_IMPORT("html", "first") std_obj_t html_first(std_obj_t obj);
WASM_IMPORT("html", "last") std_obj_t html_last(std_obj_t obj);
WASM_IMPORT("html", "array") std_obj_t html_array(std_obj_t obj);

WASM_IMPORT("html", "base_uri") std_obj_t html_base_uri(std_obj_t obj);
WASM_IMPORT("html", "body") std_obj_t html_body(std_obj_t obj);
WASM_IMPORT("html", "text") std_obj_t html_text(std_obj_t obj);
WASM_IMPORT("html", "html") std_obj_t html_html(std_obj_t obj);
WASM_IMPORT("html", "outer_html") std_obj_t html_outer_html(std_obj_t obj);

WASM_IMPORT("html", "id") std_obj_t html_id(std_obj_t obj);
WASM_IMPORT("html", "tag_name") std_obj_t html_tag_name(std_obj_t obj);
WASM_IMPORT("html", "class_name") std_obj_t html_class_name(std_obj_t obj);
WASM_IMPORT("html", "has_class") bool html_has_class(std_obj_t obj, char *class, size_t class_len);
WASM_IMPORT("html", "has_attr") bool html_has_attr(std_obj_t obj, char *attr, size_t attr_len);
