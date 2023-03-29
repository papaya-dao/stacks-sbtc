#!/bin/bash -e
#
# TODO: update to account for testnet/mainnet values in contract
#
############################################################
# Copies clarity contracts to test directory and filters
# out references to testnet / mainet contracts in order to 
# expose maximum test surface area.
# must be run before > clarinet test
############################################################
infile=contracts/sbtc-mini.clar
outfile=tests/contracts/sbtc-mini.clar
m1In="'ST000000000000000000002AMW42H.pox-2"
m1Out=".pox-2"

printf "Working Directory: `pwd`\n"
printf "Copying and replacing: $infile to $outfile\n"

sed 's/'$m1In'/'$m1Out'/g;' $infile > $outfile

printf "Finished!"

exit 0;
