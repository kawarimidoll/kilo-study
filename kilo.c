/*** includes ***/

#define _DEFAULT_SOURCE
#define _BSD_SOURCE
#define _GNU_SOURCE

#include <ctype.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/types.h>
#include <termios.h>
#include <unistd.h>

/*** defines ***/

#define KILO_VERSION "0.0.1"

// 0x1f represents 00011111
#define CTRL_KEY(k) ((k) & 0x1f)

enum editorKey {
  ARROW_LEFT = 1000,
  ARROW_RIGHT,
  ARROW_UP,
  ARROW_DOWN,
  DEL_KEY,
  HOME_KEY,
  END_KEY,
  PAGE_UP,
  PAGE_DOWN
};

/*** data ***/

typedef struct erow {
  int size;
  char* chars;
} erow;

struct editorConfig {
  int cx, cy;
  int rowoff;
  int screenrows, screencols;
  struct termios orig_termios;
  int numrows;
  erow* row;
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

int editorReadKey(void) {
  int nread;
  char c;
  while ((nread = read(STDIN_FILENO, &c, 1)) != 1) {
    if (nread == -1 && errno != EAGAIN) {
      die("read");
    }
  }

  if (c == '\x1b') {
    /* escape sequences */
    char seq[3];
    if (read(STDIN_FILENO, &seq[0], 1) == 1 &&
        read(STDIN_FILENO, &seq[1], 1) == 1) {
      if (seq[0] == '[') {
        if ('0' <= seq[1] && seq[1] <= '9') {
          if (read(STDIN_FILENO, &seq[2], 1) == 1 && seq[2] == '~') {
            switch (seq[1]) {
              case '3':
                return DEL_KEY;
              case '1':
              case '7':
                return HOME_KEY;
              case '4':
              case '8':
                return END_KEY;
              case '5':
                return PAGE_UP;
              case '6':
                return PAGE_DOWN;
            }
          }
        } else {
          switch (seq[1]) {
            case 'A':
              return ARROW_UP;
            case 'B':
              return ARROW_DOWN;
            case 'C':
              return ARROW_RIGHT;
            case 'D':
              return ARROW_LEFT;
            case 'H':
              return HOME_KEY;
            case 'F':
              return END_KEY;
          }
        }
      } else if (seq[0] == 'O') {
        switch (seq[1]) {
          case 'H':
            return HOME_KEY;
          case 'F':
            return END_KEY;
        }
      }
    }

    return '\x1b';
  }
  return c;
}

int getCursorPosition(int* rows, int* cols) {
  char buf[32];
  unsigned int i = 0;

  /* query cursor position */
  if (write(STDOUT_FILENO, "\x1b[6n", 4) != 4) {
    return -1;
  }

  while (i < sizeof(buf) - 1) {
    if (read(STDIN_FILENO, &buf[i], 1) != 1) {
      break;
    }
    if (buf[i] == 'R') {
      break;
    }
    i++;
  }
  buf[i] = '\0';

  /* make sure it responded with an escape sequence */
  if (buf[0] != '\x1b' || buf[1] != '[') {
    return -1;
  }
  if (sscanf(&buf[2], "%d;%d", rows, cols) != 2) {
    return -1;
  }

  /* to check */
  /* printf("%d, %d \r\n", *rows, *cols); */
  /* printf("hoge \r\n"); */
  /* editorReadKey(); */

  return 0;
}

int getWindowSize(int* rows, int* cols) {
  struct winsize ws;
  if (ioctl(STDOUT_FILENO, TIOCGWINSZ, &ws) == -1 || ws.ws_col == 0) {
    /* put cursor on the right bottom of current screen */
    if (write(STDOUT_FILENO, "\x1b[999C\x1b[999B", 12) != 12) {
      return -1;
    }
    return getCursorPosition(rows, cols);
  }
  *cols = ws.ws_col;
  *rows = ws.ws_row;
  return 0;
}

/*** row operations ***/

void editorAppendRow(char* s, size_t len) {
  E.row = realloc(E.row, sizeof(erow) * (E.numrows + 1));

  int at = E.numrows;
  E.row[at].size = len;
  E.row[at].chars = malloc(len + 1);
  memcpy(E.row[at].chars, s, len);
  E.row[at].chars[len] = '\0';
  E.numrows++;
}

/*** file io ***/

void editorOpen(char* filename) {
  FILE* fp = fopen(filename, "r");
  if (!fp) {
    die("fopen");
  }

  char* line = NULL;
  size_t linecap = 0;
  ssize_t linelen;

  while ((linelen = getline(&line, &linecap, fp)) != -1) {
    while (linelen > 0 &&
           (line[linelen - 1] == '\n' || line[linelen - 1] == '\r')) {
      linelen--;
    }

    editorAppendRow(line, linelen);
  }
  free(line);
  fclose(fp);
}

/*** append buffer ***/

struct abuf {
  char* b;
  int len;
};
#define ABUF_INIT {NULL, 0}

void abAppend(struct abuf* ab, const char* s, int len) {
  char* new = realloc(ab->b, ab->len + len);

  if (new == NULL) {
    return;
  }
  memcpy(&new[ab->len], s, len);
  ab->b = new;
  ab->len += len;
}

void abFree(struct abuf* ab) {
  free(ab->b);
}

/*** output ***/

void editorScroll(void) {
  if (E.cy < E.rowoff) {
    E.rowoff = E.cy;
  }
  if (E.cy >= E.rowoff + E.screenrows) {
    E.rowoff = E.cy - E.screenrows + 1;
  }
}

void editorDrawRows(struct abuf* ab) {
  int i;
  for (i = 0; i < E.screenrows - 1; i++) {
    int filerow = i + E.rowoff;
    if (filerow >= E.numrows) {
      /* display welcome message when file is NOT opened (E.numrows==0) */
      if (E.numrows == 0 && i == E.screenrows / 2) {
        char welcome[80];
        int welcomelen = snprintf(welcome, sizeof(welcome),
                                  "Kilo editor -- version %s", KILO_VERSION);
        if (welcomelen > E.screencols) {
          welcomelen = E.screencols;
        }
        int padding = (E.screencols - welcomelen) / 2;
        if (padding) {
          abAppend(ab, "~", 1);
          padding--;
        }
        while (padding--) {
          abAppend(ab, " ", 1);
        }

        abAppend(ab, welcome, welcomelen);
      } else {
        abAppend(ab, "~", 1);
      }
    } else {
      int len = E.row[filerow].size;
      if (len > E.screencols) {
        len = E.screencols;
      }
      abAppend(ab, E.row[filerow].chars, len);
    }
    /* <esc>K to clear the right part of the current line */
    abAppend(ab, "\x1b[K\r\n", 5);
  }
  /* do not put newline on the last line to avoid to scroll */
  abAppend(ab, "~\x1b[K", 4);
}

void editorRefreshScreen(void) {
  editorScroll();

  struct abuf ab = ABUF_INIT;

  /* hide cursor */
  abAppend(&ab, "\x1b[?25l", 6);
  /* set cursor to origin */
  abAppend(&ab, "\x1b[H", 3);

  editorDrawRows(&ab);

  char buf[32];
  /* need +1 because cx/cy is 0-origin, cursor in screen is 1-origin */
  snprintf(buf, sizeof(buf), "\x1b[%d;%dH", (E.cy - E.rowoff) + 1, E.cx + 1);
  abAppend(&ab, buf, strlen(buf));

  /* show cursor */
  abAppend(&ab, "\x1b[?25h", 6);

  write(STDOUT_FILENO, ab.b, ab.len);
  abFree(&ab);
}

/*** input ***/

void editorMoveCursor(int key) {
  switch (key) {
    case ARROW_UP:
      if (E.cy != 0) {
        E.cy--;
      }
      break;
    case ARROW_DOWN:
      if (E.cy < E.numrows) {
        E.cy++;
      }
      break;
    case ARROW_LEFT:
      if (E.cx != 0) {
        E.cx--;
      }
      break;
    case ARROW_RIGHT:
      if (E.cy != E.screencols - 1) {
        E.cx++;
      }
      break;
  }
}

int editorProcessKeypress(void) {
  int c = editorReadKey();
  switch (c) {
    case 'q':
    case CTRL_KEY('q'):
      return 1;

    case HOME_KEY:
      E.cx = 0;
      break;
    case END_KEY:
      E.cx = E.screencols - 1;
      break;

    case PAGE_UP:
    case PAGE_DOWN: {
      int times = E.screenrows;
      while (times--) {
        editorMoveCursor(c == PAGE_UP ? ARROW_UP : ARROW_DOWN);
      }
    } break;

    case 'h':
      editorMoveCursor(ARROW_LEFT);
      break;
    case 'l':
      editorMoveCursor(ARROW_RIGHT);
      break;
    case 'k':
      editorMoveCursor(ARROW_UP);
      break;
    case 'j':
      editorMoveCursor(ARROW_DOWN);
      break;

    case ARROW_LEFT:
    case ARROW_RIGHT:
    case ARROW_UP:
    case ARROW_DOWN:
      editorMoveCursor(c);
      break;
  }
  return 0;
}

/*** init ***/

void initEditor(void) {
  E.cx = 0;
  E.cy = 0;
  E.rowoff = 0;
  E.numrows = 0;
  E.row = NULL;

  if (getWindowSize(&E.screenrows, &E.screencols) == -1) {
    die("getWindowSize");
  }
}

int main(int argc, char* argv[]) {
  printf("start kilo\r\n");
  enableRawMode();
  initEditor();
  if (argc >= 2) {
    editorOpen(argv[1]);
  }

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
