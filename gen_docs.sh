#!/bin/bash

cargo doc --target=aarch64-unknown-none
cp -r $PWD/target/aarch64-unknown-none/doc/ docs