# Rust CI Dockerfile (librpm-rs project)
#
# Resulting image is published as rustrpm/ci on Docker Hub

FROM quay.io/centos:stream9

# Update container RPMs and install Rust compiler + rust dev tools
RUN dnf --assumeyes update && \
    dnf --assumeyes install rust cargo clippy rustfmt clang-devel rpm-devel zlib-devel && \
    dnf clean all

# Configure Rust environment variables
ENV RUST_BACKTRACE full
