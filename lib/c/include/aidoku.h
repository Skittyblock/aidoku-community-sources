#include <stdbool.h>
#include "wasm.h"

typedef unsigned long size_t;

typedef enum {
	STATUS_UNKNOWN = 0,
	STATUS_ONGOING = 1,
	STATUS_COMPLETED = 2,
	STATUS_CANCELLED = 3,
	STATUS_HIATUS = 4,
} status_type_t;

typedef enum {
	CONTENT_SAFE = 0,
	CONTENT_SUGGESTIVE = 1,
	CONTENT_NSFW = 2,
} content_type_t;

typedef enum {
	VIEWER_RTL = 1,
	VIEWER_LTR = 2,
	VIEWER_VERTICAL = 3,
	VIEWER_VERTICAL_SCROLL = 4,
} manga_viewer_type_t;

WASM_IMPORT("aidoku", "create_manga")
int create_manga(
	char *id, size_t id_len,
	char *cover_url, size_t cover_url_len,
	char *title, size_t title_len,
	char *author, size_t author_len,
	char *artist, size_t artist_len,
	char *description, size_t description_len,
	char *url, size_t url_len,
	char **categories, int *category_str_lens, size_t category_count,
	status_type_t status, content_type_t nsfw, manga_viewer_type_t viewer
);

WASM_IMPORT("aidoku", "create_manga_result") int create_manga_result(int manga_arr, bool has_more);

WASM_IMPORT("aidoku", "create_chapter")
int create_chapter(
	char *id, int id_len,
	char *title, size_t title_len,
	float volume, float chapter, double dateUpdated,
	char *scanlator, size_t scanlator_len,
	char *url, size_t url_len,
	char *lang, size_t lang_len
);

WASM_IMPORT("aidoku", "create_page")
int create_page(
	int index,
	char *image_url, size_t image_url_len,
	char *base64, size_t base64_len,
	char *text, size_t text_len
);
