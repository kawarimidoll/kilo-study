/*** includes ***/

#include <ctype.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <termios.h>
#include <unistd.h>

/*** defines ***/

// 0x1f represents 00011111
#define CTRL_KEY(k) ((k) & 0x1f)

/*** data ***/

struct termios orig_termios;

/*** terminal ***/

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

char editorReadKey(void) {
  int nread;
  char c;
  while ((nread = read(STDIN_FILENO, &c, 1)) == 1) {
    if (nread == -1 && errno != EAGAIN) {
      die("read");
    }
  }
  return c;
}

/*** input ***/

int editorProcessKeypress(void) {
  char c = editorReadKey();
  switch (c) {
    case 'q':
    case CTRL_KEY('q'):
      return 1;
  }
  return 0;
}

/*** init ***/

int main(void) {
  printf("start kilo\r\n");
  enableRawMode();

  while (1) {
    if (editorProcessKeypress()) {
      printf("quit\r\n");
      break;
    }
  }

  printf("end kilo\r\n");
  return 0;
}
