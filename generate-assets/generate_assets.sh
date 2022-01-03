#!/bin/sh

git clone https://github.com/BlackPhlox/bevy-assets assets 

cargo run --bin generate -- assets ../content