FROM ubuntu:22.04

ARG DEBIAN_FRONTEND=noninteractive
ARG HOST_UID=1000
ARG HOST_GID=1000

RUN apt-get update && apt-get install -y --no-install-recommends \
    git openssh-client make gcc patch libssl-dev liblz4-tool expect expect-dev g++ \
    patchelf chrpath gawk texinfo diffstat binfmt-support qemu-user-static \
    live-build bison flex fakeroot cmake gcc-multilib g++-multilib unzip \
    device-tree-compiler libncurses-dev libgucharmap-2-90-dev bzip2 expat \
    gpgv2 cpp-aarch64-linux-gnu libgmp-dev libmpc-dev bc python-is-python3 \
    python2 python3-pip curl file rsync bsdmainutils scons ca-certificates locales xz-utils \
    && python3 -m pip install --no-cache-dir gdown \
    && locale-gen en_US.UTF-8 \
    && ln -sf /usr/bin/python2 /usr/local/bin/python \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --gid "${HOST_GID}" builder \
    && useradd --uid "${HOST_UID}" --gid "${HOST_GID}" --create-home --shell /bin/bash builder

ENV LANG=en_US.UTF-8
ENV LC_ALL=en_US.UTF-8
ENV FORCE_UNSAFE_CONFIGURE=1

USER builder
WORKDIR /sdk
CMD ["bash"]
