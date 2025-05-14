#!/bin/bash

needs -vvv eza bat btm
cargo run -- -vvv eza bat btm

echo "Everything ok!"

cargo run -- -vvv somethingelse || {
  echo "Somethingelse is not installed"
  exit 1
}
