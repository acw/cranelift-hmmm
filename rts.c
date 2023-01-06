#include <stdint.h>
#include <stdio.h>

void print(char *variable_name, uint64_t value) {
    printf("%s = %llu\n", variable_name, value);
}

void caller() {
    print("x", 4);
}

extern void gogogo();

int main(int argc, char **argv) {
    gogogo();
    return 0;
}