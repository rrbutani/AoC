#!/bin/bash

# $1 is input string.

for ((i=s=0; i < ${#1}; i++)); do
    [ ${1:i:1} -eq ${1:((i+1)%${#1}):1} ] && ((s+=${1:i:1}))
done

echo "P1: $s"


for ((i=s=0; i < ${#1}; i++)); do
    [ ${1:i:1} -eq ${1:((i+(${#1}/2))%${#1}):1} ] && ((s+=${1:i:1}))
done

echo "P1: $s"

##############################
# Author: Rahul Butani       #
# Date:   December 1st, 2017 #
##############################