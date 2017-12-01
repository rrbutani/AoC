#include <stdio.h>

int p1(int argc, char **argv)
{
    unsigned long sum = 0;
    char *input = argv[1];

    while(*input != '\0')
        sum += (*input == *++input) * (*input - '0');

    sum += (*--input == *argv[1]) * (*input - '0');

    printf("%lu\n", sum);
}

int p2(char *inp)
{
    unsigned long sum = 0, len = 0;

    while(inp[len] != '\0') len++;

    for(unsigned long i = 0; i < len; i++)
        sum += (inp[i] == inp[(i + (len/2)) % len]) * (inp[i] - '0');

    printf("%lu\n", sum);
}

int main(int argc, char **argv)
{
    p2(argv[1]);
}