#!/bin/bash
cargo build --release

target=riscv64gc-unknown-none-elf
build=release
elf=vf2-riscv-rt

#Build the path to the elf object file
elfpath=target/$target/$build/$elf

#Dump the object file
objdmppath=$elf.objdmp
riscv64-unknown-elf-objdump -CdS $elfpath > $objdmppath

#Convert elf to binary
binpath=$elfpath.bin
riscv64-unknown-elf-objcopy -O binary --strip-all $elfpath $binpath

#Add spl header information
splbinpath=$binpath.normal.out
../../../riscv/Tools/spl_tool/spl_tool -c -f $binpath $splbinpath

#Compile the device tree
#dtc -I dts -O dtb -o board.dtb board.dts

#Append it to the splbin
#cat board.dtb >> $splbinpath

