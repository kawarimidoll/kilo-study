#include <stdio.h>
#include <termios.h>
#include <unistd.h>

void enableRawMode(void) {
  struct termios raw;
  tcgetattr(STDIN_FILENO, &raw);
  raw.c_lflag &= ~(ECHO);
  tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw);
}

int main(void) {
  printf("start kilo\n");
  enableRawMode();

  char c;

  while (read(STDIN_FILENO, &c, 1) == 1 && c != 'q')
    ;

  printf("end kilo\n");
  return 0;
}
