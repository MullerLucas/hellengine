#!/bin/env bash

shader_file="$1"
echo "> start compiling shaders '$shader_file'...."

sh_dir=$(dirname $0)

vert_in="${shader_file}.vert"
frag_in="${shader_file}.frag"

[ -f "$vert_in" ] || { echo "> invalid vertex input file '$vert_in'" ; exit 1 ; }
[ -f "$frag_in" ] || { echo "> invalid fragment input file '$frag_in'" ; exit 1 ; }

vert_out="${vert_in}.spv"
frag_out="${frag_in}.spv"

glslc "$vert_in" -o "$vert_out"
glslc "$frag_in" -o "$frag_out"

echo "> done compiling shaders...."
