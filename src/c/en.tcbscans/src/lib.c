void *memcpy(void *dest, const void *src, unsigned long n) {
   char *csrc = (char *)src;
   char *cdest = (char *)dest;
  
   for (int i=0; i<n; i++) cdest[i] = csrc[i];

   return cdest;
}

int strcmp(const char *x, const char *y) {
	while (*x) {
		if (*x != *y) {
			break;
		}
		x++;
		y++;
	}
	return *(const unsigned char *)x - *(const unsigned char *)y;
}

float power(float x, int y) {
	float p = 1;
	float xx = x;
	int i;
	if (y < 0) {
		y =- 1 * y;
		xx = 1 / xx;
	}
	for (int i = 1; i <= y; i++) {
		p = p * xx;
	}
	return p;
}

double atof(char *arr) {
	int i, j, flag;
	double val;
	char c;
	i = 0;
	j = 0;
	val = 0;
	flag = 0;
	while ((c = *(arr + i)) != '\0') {
		if (c != '.') {
			val = (val * 10) + (c - '0');
			if (flag == 1){
				--j;
			}
		}
		if (c == '.') {
			if (flag == 1) return 0;
			flag = 1;
		}
		++i;
	}
	val = val * power(10, j);
	return val;
}

// Get substring after last occurrence of deliminator
char *findlast(char *str, char delim) {
	char *c = str;
	char *p = 0;
	while (*c) {
		if (*c == delim) p = c;
		++c;
	}
	if (p) {
		if (*(p + 1)) return p + 1;
		else return 0;
	} else {
		return p;
	}
}
