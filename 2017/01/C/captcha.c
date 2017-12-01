#include <stdio.h>

void p1(char *inp)
{
    unsigned long sum = 0;
    char *in = inp;

    while(*in != '\0')
        sum += (*in == *++in) * (*in - '0');

    sum += (*--in == *inp) * (*in - '0');

    printf("P1: %lu\n", sum);
}

void p2(char *inp)
{
    unsigned long sum = 0, len = 0;

    while(inp[++len] != '\0');

    for(unsigned long i = 0; i < len; i++)
        sum += (inp[i] == inp[(i + (len/2)) % len]) * (inp[i] - '0');

    printf("P2: %lu\n", sum);
}

int main(int argc, char **argv)
{
    p1(argv[1]);

    p2(argv[1]);
}