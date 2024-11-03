#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct Point {
  double x;
  double y;
} Point;

struct Point *create_point(double x, double y);

double get_x(const struct Point *point);

double get_y(const struct Point *point);

void free_point(void *point);
