#!/bin/bash

cargo doc --target=aarch64-nintendo-switch-freestanding --release --features services,crypto,smc,gpu,fs,input,la,rand
cp -r $PWD/target/aarch64-nintendo-switch-freestanding/doc/ docs