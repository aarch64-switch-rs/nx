#!/bin/bash

cargo doc --target=aarch64-nintendo-switch-freestanding --release --features services
cp -r $PWD/target/aarch64-nintendo-switch-freestanding/doc/ docs
