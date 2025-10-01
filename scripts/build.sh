#!/bin/bash

rm -rf abi && mkdir -p abi/contracts
rm -rf typescript-types && mkdir typescript-types
rm -rf dist && mkdir -p dist/commands

tsc || true

for dir in ./contracts/*/; do
  subdir=$(echo $dir | cut -d'/' -f2)
  cd $dir
  yarn
  # npx hardhat compile --force
  ls
  tsc || true
  # cp -r artifacts/contracts/* ../../abi/$subdir/
  cd ../../
done

find ./abi/ -name '*.dbg.json' -exec rm {} \;