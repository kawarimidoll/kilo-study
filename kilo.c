#include <stdio.h>
#include <unistd.h>

int main(void) {
  printf("start kilo\n");
  char c;

  while (read(STDIN_FILENO, &c, 1) == 1)
    ;

  printf("end kilo\n");
  return 0;
}
