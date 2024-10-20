# vf2-riscv-rt
Personal project playing with the vision five 2.

Binary is loaded and executed via xmodem when the board is set to boot from
uart.

mkimg.sh is crude script to build the project, dump an object file and add an
spl header.  Right now I am using tio for uart communication and loading the
binary to the vf2.
