#!/bin/bash

in_dir="shaders/glsl"
out_dir="shaders/spv"

[ -d "$out_dir" ] || mkdir -p "$out_dir"

glslc "$in_dir"/triangle.vert -o "$out_dir"/triangle_vert.spv
glslc "$in_dir"/triangle.frag -o "$out_dir"/triangle_frag.spv
