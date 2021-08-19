#!/bin/sh

git clone https://github.com/BlackPhlox/bevy-assets assets 

cargo run --bin generate -- assets ../content

#read -n1 -rsp $'Press any key to continue or Ctrl+C to exit...\n'