#include <unistd.h>
#include <stdio.h>
int main()
{
    const int rc = execve("/bin/ls", (char *[]){"ls", "-l", NULL}, NULL);
    printf("%d\n", rc);
    return 0;
}
