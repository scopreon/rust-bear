FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

# Base system + C toolchain
RUN apt-get update && apt-get install -y \
    build-essential \
    clang \
    lldb \
    gdb \
    cmake \
    make \
    pkg-config \
    strace \
    ltrace \
    git \
    curl \
    ca-certificates \
    vim \
    nano \
    less \
    && rm -rf /var/lib/apt/lists/*

# Install Rust (stable)
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

# Rust tooling useful for systems work
RUN rustup component add \
    rustfmt \
    clippy \
    rust-src

# Set working directory
WORKDIR /workspace

# Default shell
CMD ["/bin/bash"]
