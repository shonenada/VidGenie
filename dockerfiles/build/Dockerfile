FROM rust:slim-bookworm

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libgstreamer1.0-dev \
        libgstreamer-plugins-base1.0-dev \
        libgstreamer-plugins-bad1.0-dev \
        gstreamer1.0-plugins-base \
        gstreamer1.0-plugins-good \
        gstreamer1.0-plugins-bad \
        gstreamer1.0-plugins-ugly \
        gstreamer1.0-libav \
        gstreamer1.0-tools \
        gstreamer1.0-alsa \
        gstreamer1.0-gl \
        gstreamer1.0-pulseaudio \
        make cmake gcc g++ \
        pkg-config \
        libssl-dev \
        libfontconfig1-dev \
        libfreetype6 \
        libfreetype6-dev \
        libegl1-mesa-dev \
        libgbm-dev && \
    rm -rf /var/lib/apt/lists/*

# Pre-download and cache cargo dependencies
WORKDIR /tmp/vidgenie-deps
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY src/vg-gl/Cargo.toml src/vg-gl/
COPY src/vg-gst/Cargo.toml src/vg-gst/
COPY src/vg-video/Cargo.toml src/vg-video/
COPY src/vg-cli/Cargo.toml src/vg-cli/

# Create dummy source files to allow cargo fetch/build to work
RUN mkdir -p src/vg-gl/src src/vg-gst/src src/vg-video/src src/vg-cli/src/bin && \
    echo "fn main() {}" > src/vg-cli/src/bin/vidgenie.rs && \
    touch src/vg-gl/src/lib.rs src/vg-gst/src/lib.rs src/vg-video/src/lib.rs

# Fetch all dependencies and build to cache compiled crates
RUN cargo fetch && \
    cargo build --release -p vg-cli 2>/dev/null || true && \
    rm -rf src

WORKDIR /workspace

ENV VIDGENIE_HEADLESS=1
