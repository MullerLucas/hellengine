#!/bin/env bash

POSITIONAL_ARGS=()

while [[ $# -gt 0 ]]; do
  case $1 in
    -c|--check)
      SHOULD_CHECK=YES
      shift # past value
      ;;
    -r|--run)
      SHOULD_RUN=YES
      shift # past argument
      ;;
    -*|--*)
      echo "Unknown option $1"
      exit 1
      ;;
    *)
      POSITIONAL_ARGS+=("$1") # save positional arg
      shift # past argument
      ;;
  esac
done


echo "SHOULD_CHECK: '$SHOULD_CHECK'"
echo "  SHOULD_RUN: '$SHOULD_RUN'"

# -----------------------------------------------


# cargo-clippy

if [ "$SHOULD_CHECK" = "YES" ]; then
    echo "TEST: '$SHOULD_CHECK'"
    echo "start checking project..."
    cargo check
elif [ "$SHOULD_RUN" = "YES" ]; then
    echo "start running project..."
    cargo run
else
    echo "start building project..."
    cargo build
fi
