# Rust CI Dockerfile (librpm-rs project)
#
# Resulting image is published as rustrpm/ci on Docker Hub

FROM centos:7.4.1708

# Include cargo in the path
ENV PATH "$PATH:/root/.cargo/bin"

# Install/update RPMs
RUN yum update -y && \
    yum groupinstall -y "Development Tools" && \
    yum install -y centos-release-scl rpm-devel && \
    yum install -y --enablerepo=centos-sclo-rh llvm-toolset-7

# Set environment variables to enable llvm-toolset-7 SCL package
ENV LD_LIBRARY_PATH "/opt/rh/llvm-toolset-7/root/usr/lib64"
ENV PATH "/opt/rh/llvm-toolset-7/root/usr/bin:/opt/rh/llvm-toolset-7/root/usr/sbin:$PATH"
ENV PKG_CONFIG_PATH "/opt/rh/llvm-toolset-7/root/usr/lib64/pkgconfig"
ENV X_SCLS llvm-toolset-7

# Install rustup
WORKDIR /root
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Rust nightly version to install
ENV RUST_NIGHTLY_VERSION "nightly-2018-06-02"

# Install Rust nightly
RUN rustup install $RUST_NIGHTLY_VERSION

# Add the Rust sysroot to ld.so.conf
RUN bash -l -c "echo $(rustc --print sysroot)/lib >> /etc/ld.so.conf"
RUN ldconfig

# Install rustfmt
RUN rustup component add rustfmt-preview --toolchain $RUST_NIGHTLY_VERSION

# Install clippy
ENV CLIPPY_VERSION "0.0.206"
RUN cargo +$RUST_NIGHTLY_VERSION install clippy --vers $CLIPPY_VERSION

# Configure Rust environment variables
ENV RUST_BACKTRACE full
