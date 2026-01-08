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
        gstreamer1.0-x \
        gstreamer1.0-alsa \
        gstreamer1.0-gl \
        gstreamer1.0-gtk3 \
        gstreamer1.0-qt5 \
        gstreamer1.0-pulseaudio \
        cmake gcc g++ \
        pkg-config \
        libssl-dev \
        libfontconfig1-dev \
        libxrandr-dev \
        libxinerama-dev \
        libxcursor-dev \
        libxi-dev \
        libfreetype6 \
        libfreetype6-dev \
        xvfb xauth && \
    rm -rf /var/lib/apt/lists/*

# xvfb-run -s "-screen 0 1280x720x24" cargo run -p vg-cli -- --file examples/transition_fade_test.json
