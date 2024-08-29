#include <ctype.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <termios.h>
#include <unistd.h>

struct termios orig_termios;

void die(const char* s) {
  perror(s);
  exit(1);
}

void disableRawMode(void) {
  if (tcsetattr(STDIN_FILENO, TCSAFLUSH, &orig_termios) == -1) {
    die("tcsetattr");
  }
}

void enableRawMode(void) {
  if (tcgetattr(STDIN_FILENO, &orig_termios) == -1) {
    die("tcgetattr");
  }
  atexit(disableRawMode);

  struct termios raw = orig_termios;
  raw.c_iflag &= ~(BRKINT | INPCK | ISTRIP | ICRNL | IXON);
  raw.c_oflag &= ~(OPOST);
  raw.c_cflag |= CS8;
  raw.c_lflag &= ~(ECHO | ICANON | IEXTEN | ISIG);
  raw.c_cc[VMIN] = 0;
  raw.c_cc[VTIME] = 1;  // 1/10 second

  if (tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw) == -1) {
    die("tcsetattr");
  }
}

int main(void) {
  printf("start kilo\r\n");
  enableRawMode();

  while (1) {
    char c = '\0';

    int read_out = read(STDIN_FILENO, &c, 1);
    if (read_out == -1 && errno != EAGAIN) {
      die("read");
    }
    /* printf("read_out: %d\r\n", read_out); */
    if (read_out == 0) {
      // no input
    } else {
      if (iscntrl(c)) {
        printf("%d\r\n", c);
      } else {
        printf("%d ('%c')\r\n", c, c);
      }
      if (c == 'q') {
        printf("quit\r\n");
        break;
      }
    }
  }

  printf("end kilo\r\n");
  return 0;
}
