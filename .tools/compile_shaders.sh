#!/bin/env bash


vert_in=shaders/sprite.vert
frag_in=shaders/sprite.frag

glslc "$vert_in" -o "$vert_in".spv
glslc "$frag_in" -o "$frag_in".spv
