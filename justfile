set shell := ["sh", "-cu"]

default: build

build:
    cargo build --release
    ln -sfn target/release/imagegen imagegen
