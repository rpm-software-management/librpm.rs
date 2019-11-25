# Rust CI Dockerfile (librpm-rs project)
#
# Resulting image is published as rustrpm/ci on Docker Hub

FROM centos:7.7.1908

RUN yum update -y && \
# Binding target
    yum install -y rpm-devel && \
# Development tools
    yum install -y gcc clang llvm-devel gdb valgrind

RUN debuginfo-install -y rpm-libs-4.11.3-40.el7.x86_64

RUN cd /tmp && \
    curl https://static.rust-lang.org/dist/rust-1.39.0-x86_64-unknown-linux-gnu.tar.gz > rust.tar.gz && \
    tar -xaf rust.tar.gz && \
    cd rust* && \
    ./install.sh

# Configure Rust environment variables
ENV RUST_BACKTRACE full
