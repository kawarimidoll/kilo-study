/*** includes ***/

#define _DEFAULT_SOURCE
#define _BSD_SOURCE
#define _GNU_SOURCE

#include <ctype.h>
#include <errno.h>
#include <fcntl.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/types.h>
#include <termios.h>
#include <time.h>
#include <unistd.h>

/*** defines ***/

#define KILO_VERSION "0.0.1"
#define KILO_TAB_STOP 8

// 0x1f represents 00011111
#define CTRL_KEY(k) ((k) & 0x1f)

enum editorKey {
  BACKSPACE = 127,
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
  int rsize;
  char* chars;
  char* render;
} erow;

struct editorConfig {
  int cx, cy;
  int rx;
  int rowoff, coloff;
  int screenrows, screencols;
  struct termios orig_termios;
  int numrows;
  int dirty;
  char* filename;
  char statusmsg[80];
  time_t statusmsg_time;
  erow* row;
};
struct editorConfig E;

/*** prototypes ***/

void editorSetStatusMessage(const char* fmt, ...);

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

int editorRowCxToRx(erow* row, int cx) {
  int rx = 0;
  int j;
  for (j = 0; j < cx; j++) {
    if (row->chars[j] == '\t') {
      rx += (KILO_TAB_STOP - 1) - (j % KILO_TAB_STOP);
    }
    rx++;
  }

  return rx;
}

void editorUpdateRow(erow* row) {
  int j;
  int tabs = 0;
  for (j = 0; j < row->size; j++) {
    if (row->chars[j] == '\t') {
      tabs++;
    }
  }

  free(row->render);
  row->render = malloc(row->size + tabs * (KILO_TAB_STOP - 1) + 1);

  int idx = 0;
  for (j = 0; j < row->size; j++) {
    if (row->chars[j] == '\t') {
      row->render[idx++] = ' ';
      while (idx % KILO_TAB_STOP != 0) {
        row->render[idx++] = ' ';
      }
    } else {
      row->render[idx++] = row->chars[j];
    }
  }
  row->render[idx] = '\0';
  row->rsize = idx;
}

void editorAppendRow(char* s, size_t len) {
  E.row = realloc(E.row, sizeof(erow) * (E.numrows + 1));

  int at = E.numrows;
  E.row[at].size = len;
  E.row[at].chars = malloc(len + 1);
  memcpy(E.row[at].chars, s, len);
  E.row[at].chars[len] = '\0';

  E.row[at].rsize = 0;
  E.row[at].render = NULL;
  editorUpdateRow(&E.row[at]);

  E.numrows++;
  E.dirty++;
}

void editorRowInsertChar(erow* row, int at, int c) {
  if (at < 0 || at > row->size) {
    at = row->size;
  }
  /* add 2 because we need allocate new character and null byte */
  row->chars = realloc(row->chars, row->size + 2);
  memmove(&row->chars[at + 1], &row->chars[at], row->size - at + 1);
  row->size++;
  row->chars[at] = c;
  editorUpdateRow(row);
  E.dirty++;
}

/*** editor operations ***/

void editorInsertChar(int c) {
  if (E.cy == E.numrows) {
    /* append line */
    editorAppendRow("", 0);
  }
  editorRowInsertChar(&E.row[E.cy], E.cx, c);
  E.cx++;
}

/*** file io ***/

char* editorRowsToString(int* buflen) {
  int totlen = 0;
  int j;
  for (j = 0; j < E.numrows; j++) {
    totlen += E.row[j].size + 1;
  }
  *buflen = totlen;

  char* buf = malloc(totlen);
  char* p = buf;
  for (j = 0; j < E.numrows; j++) {
    memcpy(p, E.row[j].chars, E.row[j].size);
    p += E.row[j].size;
    *p = '\n';
    p++;
  }

  return buf;
}

void editorOpen(char* filename) {
  free(E.filename);
  E.filename = strdup(filename);

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
  E.dirty = 0;
}

void editorSave(void) {
  if (E.filename == NULL) {
    return;
  }

  int len;
  char* buf = editorRowsToString(&len);

  int fd = open(E.filename, O_RDWR | O_CREAT, 0644);
  if (fd != -1) {
    if (ftruncate(fd, len) != -1) {
      if (write(fd, buf, len) == len) {
        close(fd);
        free(buf);
        editorSetStatusMessage("%d bytes written to disk", len);
        E.dirty = 0;
        return;
      }
    }
    close(fd);
  }
  free(buf);
  editorSetStatusMessage("Can't save! I/O error: %s", strerror(errno));
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
  E.rx = 0;
  if (E.cy < E.numrows) {
    E.rx = editorRowCxToRx(&E.row[E.cy], E.cx);
  }

  if (E.cy < E.rowoff) {
    E.rowoff = E.cy;
  }
  if (E.cy >= E.rowoff + E.screenrows) {
    E.rowoff = E.cy - E.screenrows + 1;
  }
  if (E.rx < E.coloff) {
    E.coloff = E.rx;
  }
  if (E.rx >= E.coloff + E.screencols) {
    E.coloff = E.rx - E.screencols + 1;
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
      int len = E.row[filerow].rsize - E.coloff;
      if (len < 0) {
        len = 0;
      }
      if (len > E.screencols) {
        len = E.screencols;
      }
      abAppend(ab, &E.row[filerow].render[E.coloff], len);
    }
    /* <esc>K to clear the right part of the current line */
    abAppend(ab, "\x1b[K\r\n", 5);
  }
  /* do not put newline on the last line to avoid to scroll */
  /* abAppend(ab, "~\x1b[K", 4); */
}

void editorDrawStatusBar(struct abuf* ab) {
  abAppend(ab, "\x1b[7m", 4);
  char status[80], rstatus[80];
  int len = snprintf(status, sizeof(status), "%.20s - %d lines %s",
                     E.filename ? E.filename : "[No Name]", E.numrows,
                     E.dirty ? "(modified)" : "");
  /* add 1 since E.cy is 0-indexed */
  int rlen = snprintf(rstatus, sizeof(rstatus), "%d/%d", E.cy + 1, E.numrows);
  if (len > E.screencols) {
    len = E.screencols;
  }
  abAppend(ab, status, len);
  while (len < E.screencols) {
    if (E.screencols - len == rlen) {
      abAppend(ab, rstatus, rlen);
      break;
    } else {
      abAppend(ab, " ", 1);
      len++;
    }
  }
  abAppend(ab, "\x1b[m", 3);
  abAppend(ab, "\r\n", 2);
}

void editorDrawMessageBar(struct abuf* ab) {
  /* clear message */
  abAppend(ab, "\x1b[K", 3);
  int msglen = strlen(E.statusmsg);
  if (msglen > E.screencols) {
    msglen = E.screencols;
  }
  if (msglen && time(NULL) - E.statusmsg_time < 5) {
    abAppend(ab, E.statusmsg, msglen);
  }
}

void editorRefreshScreen(void) {
  editorScroll();

  struct abuf ab = ABUF_INIT;

  /* hide cursor */
  abAppend(&ab, "\x1b[?25l", 6);
  /* set cursor to origin */
  abAppend(&ab, "\x1b[H", 3);

  editorDrawRows(&ab);
  editorDrawStatusBar(&ab);
  editorDrawMessageBar(&ab);

  char buf[32];
  /* need +1 because cx/cy is 0-origin, cursor in screen is 1-origin */
  snprintf(buf, sizeof(buf), "\x1b[%d;%dH", (E.cy - E.rowoff) + 1,
           (E.rx - E.coloff) + 1);
  abAppend(&ab, buf, strlen(buf));

  /* show cursor */
  abAppend(&ab, "\x1b[?25h", 6);

  write(STDOUT_FILENO, ab.b, ab.len);
  abFree(&ab);
}

void editorSetStatusMessage(const char* fmt, ...) {
  va_list ap;
  va_start(ap, fmt);
  vsnprintf(E.statusmsg, sizeof(E.statusmsg), fmt, ap);
  va_end(ap);
  E.statusmsg_time = time(NULL);
}

/*** input ***/

void editorMoveCursor(int key) {
  erow* currentrow = (E.cy >= E.numrows) ? NULL : &E.row[E.cy];

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
      } else if (E.cy > 0) {
        E.cy--;
        E.cx = E.row[E.cy].size;
      }
      break;
    case ARROW_RIGHT:
      if (currentrow && E.cx < currentrow->size) {
        E.cx++;
      } else if (currentrow && E.cx == currentrow->size) {
        E.cy++;
        E.cx = 0;
      }
      break;
  }

  /* truncate cursor position after vertical move */
  currentrow = (E.cy >= E.numrows) ? NULL : &E.row[E.cy];
  int rowlen = currentrow ? currentrow->size : 0;
  if (E.cx > rowlen) {
    E.cx = rowlen;
  }
}

int editorProcessKeypress(void) {
  int c = editorReadKey();
  switch (c) {
    case '\r':
      /* TODO: insert new line */
      break;

    /* case 'q': */
    case CTRL_KEY('q'):
      return 1;

    case CTRL_KEY('s'):
      editorSave();
      break;

    case HOME_KEY:
      E.cx = 0;
      break;
    case END_KEY:
      if (E.cy < E.numrows) {
        E.cx = E.row[E.cy].size;
      }
      break;

    case BACKSPACE:
    case DEL_KEY:
    case CTRL_KEY('h'):
      /* TODO: delete character */
      break;

    case PAGE_UP:
    case CTRL_KEY('u'): {
      E.cy = E.rowoff;

      int times = E.screenrows;
      while (times--) {
        editorMoveCursor(ARROW_UP);
      }
    } break;

    case PAGE_DOWN:
    case CTRL_KEY('d'): {
      E.cy = E.rowoff + E.screenrows - 1;
      if (E.cy > E.numrows) {
        E.cy = E.numrows;
      }

      int times = E.screenrows;
      while (times--) {
        editorMoveCursor(ARROW_DOWN);
      }
    } break;

    case CTRL_KEY('b'):
      editorMoveCursor(ARROW_LEFT);
      break;
    case CTRL_KEY('f'):
      editorMoveCursor(ARROW_RIGHT);
      break;
    case CTRL_KEY('p'):
      editorMoveCursor(ARROW_UP);
      break;
    case CTRL_KEY('n'):
      editorMoveCursor(ARROW_DOWN);
      break;

    case ARROW_LEFT:
    case ARROW_RIGHT:
    case ARROW_UP:
    case ARROW_DOWN:
      editorMoveCursor(c);
      break;

    case CTRL_KEY('l'):
    case '\x1b':
      break;

    default:
      editorInsertChar(c);
      break;
  }
  return 0;
}

/*** init ***/

void initEditor(void) {
  E.cx = 0;
  E.cy = 0;
  E.rx = 0;
  E.rowoff = 0;
  E.coloff = 0;
  E.numrows = 0;
  E.dirty = 0;
  E.row = NULL;
  E.filename = NULL;
  E.statusmsg[0] = '\0';
  E.statusmsg_time = 0;

  if (getWindowSize(&E.screenrows, &E.screencols) == -1) {
    die("getWindowSize");
  }
  E.screenrows -= 2;
}

int main(int argc, char* argv[]) {
  printf("start kilo\r\n");
  enableRawMode();
  initEditor();
  if (argc >= 2) {
    editorOpen(argv[1]);
  }
  editorSetStatusMessage(
      "HELP: ctrl-q = quit, ctrl-s = save, arrow keys = move");

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
