typedef unsigned long size_t;
typedef int req_t;
typedef int std_obj_t;

typedef enum {
	REQ_GET = 0,
	REQ_POST = 1,
	REQ_HEAD = 2,
	REQ_PUT = 3,
	REQ_DELETE = 4,
} req_method_t;

WASM_IMPORT("net", "init") req_t request_init(req_method_t method);
WASM_IMPORT("net", "send") void request_send(req_t req);
WASM_IMPORT("net", "close") void request_close(req_t req);

WASM_IMPORT("net", "set_url") void request_set_url(req_t req, char *url, size_t url_len);
WASM_IMPORT("net", "set_header") void request_set_header(req_t req, char *key, size_t key_len, char *value, size_t value_len);
WASM_IMPORT("net", "set_body") void request_set_body(req_t req, char *body, size_t body_len);

WASM_IMPORT("net", "get_url") std_obj_t request_get_url(req_t req);
WASM_IMPORT("net", "get_data") void request_get_data(req_t req, unsigned char *buffer, size_t size);
WASM_IMPORT("net", "get_data_size") size_t request_get_data_size(req_t req);

WASM_IMPORT("net", "json") std_obj_t request_json(req_t req);
WASM_IMPORT("net", "html") std_obj_t request_html(req_t req);
