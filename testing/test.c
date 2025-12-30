// #include <unistd.h>
// #include <stdio.h>
// int main()
// {
//     const int rc = execve("/bin/gcc", (char *[]){"gcc", "-l", "-a", NULL}, NULL);
//     printf("%d\n", rc);
//     return 0;
// }
#include <unistd.h>
#include <stdio.h>
#include <sys/wait.h>
#include <errno.h>

void run(char *const argv[])
{
    pid_t pid = fork();
    if (pid == 0) {
        execve("/usr/bin/gcc", argv, NULL);
        perror("execve");
        _exit(1);
    } else {
        waitpid(pid, NULL, 0);
    }
}

int main(void)
{
    run((char *[]){"gcc", "-###", "test.c", NULL});

    run((char *[]){"gcc", "-###", "a.c", "b.c", "c.c", NULL});

    run((char *[]){"gcc", "-###", "-Iinclude", "-DMODE=3", "main.c", NULL});

    run((char *[]){"gcc", "-###", "-O3", "-g", "-o", "out", "main.c", NULL});

    run((char *[]){"gcc", "-###", "test.cpp", NULL});

    run((char *[]){"gcc", "-###", "-c", "main.c", NULL});

    run((char *[]){"gcc", "-###", "--sysroot=/fake", "-mavx2", "x.c", NULL});

    run((char *[]){"gcc", "-###", "/home/user/project/src/main.c", NULL});

    return 0;
}
