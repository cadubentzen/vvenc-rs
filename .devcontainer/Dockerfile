FROM mcr.microsoft.com/devcontainers/rust:latest

RUN apt-get update && \
    apt-get install -y cmake llvm-dev libclang-dev clang && \
    apt-get clean

RUN git clone --depth 1 -b v1.13.0 https://github.com/fraunhoferhhi/vvenc.git && \
    cd vvenc && \
    make install-release-shared install-prefix=/usr && \
    rm -rf vvenc
