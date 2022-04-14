#include <stdbool.h>
#include <walloc.h>
#include <aidoku.h>
#include <std.h>
#include <net.h>
#include <html.h>

void *memcpy(void *dest, const void *src, unsigned long n);
int strcmp(const char *x, const char *y);
double atof(char *arr);
char *findlast(char *str, char delim);

static char cached_manga_id[50];
static std_obj_t cached_manga_page = -1;

void cache_manga_page(char *id, size_t id_len) {
	if (cached_manga_page != -1) destroy(cached_manga_page);

	char url[80] = "https://onepiecechapters.com";
	memcpy(&(url[28]), id, id_len);

	req_t request = request_init(REQ_GET);
	request_set_url(request, url, 28 + id_len);
	request_send(request);

	cached_manga_page = request_html(request);
	request_close(request);

	memcpy(cached_manga_id, id, id_len + 1);
}

WASM_EXPORT
std_obj_t get_manga_list(std_obj_t filter_list_obj, int page) {
	req_t request = request_init(REQ_GET);
	request_set_url(request, "https://onepiecechapters.com/projects", 37);
	request_send(request);

	std_obj_t document = request_html(request);
	request_close(request);

	std_obj_t elements = html_array(html_select(document, ".bg-card.border.border-border.rounded.p-3.mb-3", 46));

	std_obj_t manga_array = create_array();

	for (int i = 0; i < array_len(elements); i++) {
		std_obj_t element = array_get(elements, i);

		std_obj_t title_element = html_select(element, "a.mb-3.text-white.text-lg.font-bold", 35);
		std_obj_t cover_element = html_select(element, ".w-24.h-24.object-cover.rounded-lg", 34);
		std_obj_t id_text = html_attr(title_element, "href", 4);
		std_obj_t title_text = html_text(title_element);
		std_obj_t cover_text = html_attr(cover_element, "src", 3);
		
		int id_len = string_len(id_text);
		int title_len = string_len(title_text);
		int cover_len = string_len(cover_text);

		char *id = malloc(id_len);
		char *title = malloc(title_len);
		char *cover_url = malloc(cover_len);

		read_string(id_text, id, id_len);
		read_string(title_text, title, title_len);
		read_string(cover_text, cover_url, cover_len);

		array_append(
			manga_array,
			create_manga(
				id, id_len,
				cover_url, cover_len,
				title, title_len,
				"TCB Scans", 9,
				0, 0,
				0, 0,
				0, 0,
				0, 0, 0,
				STATUS_UNKNOWN, CONTENT_SAFE, VIEWER_RTL
			)
		);

		free(id);
		free(title);
		free(cover_url);
	}

	destroy(document);

	return create_manga_result(manga_array, false);
}

WASM_EXPORT
std_obj_t get_manga_details(std_obj_t manga_obj) {
	// assume max id length of 50 (current max is 36)
	char id[50];
	int id_text = object_get(manga_obj, "id", 2);
	int id_len = string_len(id_text);
	read_string(id_text, id, id_len);
	id[id_len] = '\0';

	char url[80] = "https://onepiecechapters.com";
	memcpy(&(url[28]), id, id_len);

	if (cached_manga_page == -1 || strcmp(id, cached_manga_id) != 0) {
		cache_manga_page(id, id_len);
	}

	std_obj_t element = html_select(cached_manga_page, ".order-1.bg-card.border.border-border.rounded.py-3", 50);

	std_obj_t title_text = html_text(html_select(element, ".my-3.font-bold.text-3xl", 24));
	std_obj_t desc_text = html_text(html_select(element, ".leading-6.my-3", 15));
	std_obj_t cover_text = html_attr(html_select(element, ".flex.items-center.justify-center img", 37), "src", 3);
	
	int title_len = string_len(title_text);
	int desc_len = string_len(desc_text);
	int cover_len = string_len(cover_text);

	char *title = malloc(title_len);
	char *desc = malloc(desc_len);
	char *cover_url = malloc(cover_len);

	read_string(title_text, title, title_len);
	read_string(desc_text, desc, desc_len);
	read_string(cover_text, cover_url, cover_len);

	std_obj_t result = create_manga(
		id, id_len,
		cover_url, cover_len,
		title, title_len,
		"TCB Scans", 9,
		0, 0,
		desc, desc_len,
		url, 28 + id_len,
		0, 0, 0,
		STATUS_UNKNOWN, CONTENT_SAFE, VIEWER_RTL
	);

	free(title);
	free(desc);
	free(cover_url);

	destroy(element);

	return result;
}

WASM_EXPORT
std_obj_t get_chapter_list(std_obj_t manga_obj) {
	char id[50];
	int id_text = object_get(manga_obj, "id", 2);
	int id_len = string_len(id_text);
	read_string(id_text, id, id_len);

	if (cached_manga_page == -1 || strcmp(id, cached_manga_id) != 0) {
		cache_manga_page(id, id_len);
	}

	std_obj_t elements = html_select(cached_manga_page, ".bg-card.border.border-border.rounded.p-3.mb-3", 46);
	std_obj_t elements_arr = html_array(elements);

	std_obj_t chapter_array = create_array();
	
	for (int i = 0; i < array_len(elements_arr); i++) {
		std_obj_t element = array_get(elements_arr, i);
		
		std_obj_t title_text = html_text(html_select(element, ".text-lg.font-bold:not(.flex)", 29));
		std_obj_t desc_text = html_text(html_select(element, ".text-gray-500", 14));
		std_obj_t url_text = html_text(html_attr(element, "href", 4));
	
		int title_len = string_len(title_text);
		int desc_len = string_len(desc_text);
		int url_len = string_len(url_text);

		char *title = malloc(title_len + 1);
		char *desc = malloc(desc_len);
		char *url_path = malloc(url_len);

		read_string(title_text, title, title_len);
		read_string(desc_text, desc, desc_len);
		read_string(url_text, url_path, url_len);

		title[title_len] = '\0';

		char *web_url = malloc(28 + url_len + 1);
		memcpy(web_url, "https://onepiecechapters.com", 28);
		memcpy(&(web_url[28]), url_path, url_len);

		float chapter_num = atof(findlast(title, ' '));

		array_append(
			chapter_array,
			create_chapter(
				url_path, url_len,
				desc, desc_len,
				-1, chapter_num, -1,
				"TCB Scans", 9,
				web_url, 28 + url_len,
				"en", 2
			)
		);

		free(title);
		free(desc);
		free(url_path);
		free(web_url);
	}

	destroy(elements);

	return chapter_array;
}

WASM_EXPORT
std_obj_t get_page_list(std_obj_t chapter_obj) {
	std_obj_t id_text = object_get(chapter_obj, "id", 2);
	int id_len = string_len(id_text);
	char *id = malloc(id_len);
	read_string(id_text, id, id_len);

	char *url = malloc(28 + id_len);
	memcpy(url, "https://onepiecechapters.com", 28);
	memcpy(&(url[28]), id, id_len);
	free(id);

	req_t request = request_init(REQ_GET);
	request_set_url(request, url, 28 + id_len);
	request_send(request);
	free(url);

	std_obj_t document = request_html(request);
	request_close(request);

	std_obj_t elements = html_array(html_select(document, ".flex.flex-col.items-center.justify-center picture img", 54));

	std_obj_t page_array = create_array();

	for (int i = 0; i < array_len(elements); i++) {
		std_obj_t element = array_get(elements, i);

		std_obj_t image_text = html_text(html_attr(element, "src", 3));
		int image_len = string_len(image_text);
		char *image_url = malloc(image_len);
		read_string(image_text, image_url, image_len);

		array_append(page_array, create_page(i, image_url, image_len, 0, 0, 0, 0));

		free(image_url);
	}

	destroy(document);

	return page_array;
}
