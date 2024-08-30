/*** includes ***/

#include <ctype.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/ioctl.h>
#include <termios.h>
#include <unistd.h>

/*** defines ***/

// 0x1f represents 00011111
#define CTRL_KEY(k) ((k) & 0x1f)

/*** data ***/

struct editorConfig {
  int screenrows;
  int screencols;
  struct termios orig_termios;
};
struct editorConfig E;

/*** terminal ***/

void die(const char* s) {
  write(STDOUT_FILENO, "\x1b[2J", 4);
  write(STDOUT_FILENO, "\x1b[H", 3);

  perror(s);
  exit(1);
}

void disableRawMode(void) {
  if (tcsetattr(STDIN_FILENO, TCSAFLUSH, &E.orig_termios) == -1) {
    die("tcsetattr");
  }
}

void enableRawMode(void) {
  if (tcgetattr(STDIN_FILENO, &E.orig_termios) == -1) {
    die("tcgetattr");
  }
  atexit(disableRawMode);

  struct termios raw = E.orig_termios;
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

int getWindowSize(int* rows, int* cols) {
  struct winsize ws;
  if (ioctl(STDOUT_FILENO, TIOCGWINSZ, &ws) == -1 || ws.ws_col == 0) {
    return -1;
  }
  *cols = ws.ws_col;
  *rows = ws.ws_row;
  return 0;
}

/*** output ***/

void editorRefreshScreen(void) {
  /* write to STDOUT_FILENO, 4 bytes string: "\x1b", "[", "2", "J" */
  /* "\x1b" represents escape character */
  write(STDOUT_FILENO, "\x1b[2J", 4);
  write(STDOUT_FILENO, "\x1b[H", 3);
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

void initEditor(void) {
  if (getWindowSize(&E.screenrows, &E.screencols) == -1) {
    die("getWindowSize");
  }
}

int main(void) {
  printf("start kilo\r\n");
  initEditor();
  enableRawMode();

  while (1) {
    editorRefreshScreen();
    if (editorProcessKeypress()) {
      write(STDOUT_FILENO, "\x1b[2J", 4);
      write(STDOUT_FILENO, "\x1b[H", 3);
      printf("quit\r\n");
      break;
    }
  }

  printf("end kilo\r\n");
  return 0;
}
