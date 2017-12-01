#include <stdio.h>

int main(int argc, char **argv)
{
    int a, b, c;
    unsigned long sum = 0;
    char *input = argv[1];

    c = b = *input++ - '0', a = 0;

    while(*input != '\0')
    {
        a = *input++ - '0';
        sum += (a == b) ? a : 0;
        b = a;
    }

    sum += (c == a) ? a : 0;

    printf("%lu\n", sum);
}