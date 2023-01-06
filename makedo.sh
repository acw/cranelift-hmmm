#!/bin/sh

set -e

cargo run 
gcc -g3 -o test rts.c output.o
lldb ./test