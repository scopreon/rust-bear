#include <stdio.h>

extern char _binary_mytestbinary_txt_start;
extern char _binary_mytestbinary_txt_end;

int main()
{
    for (char *c = &_binary_mytestbinary_txt_start; c < &_binary_mytestbinary_txt_end; ++c)
    {
        printf("%c", *c);
    }
}