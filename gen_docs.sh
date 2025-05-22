#!/bin/bash

cargo doc --target=aarch64-nintendo-switch-freestanding --release --features services,smc,gpu,canvas,fonts,truetype,fs,input,la,rand
cp -r $PWD/target/aarch64-nintendo-switch-freestanding/doc/ docs
