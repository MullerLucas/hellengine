#!/bin/env bash

./.tools/compile_shaders.sh || exit 1
# read -p "Shaders complied just fine (confirm):"

echo
echo "-------------------------------------------------------------------------"
echo

cargo build
