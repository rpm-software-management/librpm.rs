# Rust CI Dockerfile (librpm-rs project)
#
# Resulting image is published as rustrpm/ci on Docker Hub

FROM centos:8

# Update container RPMs and install Rust compiler + rust dev tools
RUN yum update -y && \
    yum install -y rust cargo clippy rustfmt clang-devel rpm-devel zlib-devel && \
    yum clean all

# Configure Rust environment variables
ENV RUST_BACKTRACE full
