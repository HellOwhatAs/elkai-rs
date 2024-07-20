#include "LKH.h"
#include <stdarg.h>
#include "setjmp.h"

jmp_buf ErrorJumpBuffer;
char *err_msg_buf;

void eprintf(const char *fmt, ...)
{
    va_list args;

    va_start(args, fmt);
    int res = vsprintf(err_msg_buf, fmt, args);
    va_end(args);

    longjmp(ErrorJumpBuffer, 1);
}