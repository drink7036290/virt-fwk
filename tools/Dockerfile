FROM ubuntu:24.04

ARG KERNEL_CHANNEL=v6.x
ARG KERNEL_VERSION=6.1.26
ARG BUSYBOX_VERSION=1.37.0

# persists inside the final container
# so that scripts or Makefiles can pick it up at runtime
ENV KERNEL_CHANNEL=$KERNEL_CHANNEL
ENV KERNEL_VERSION=$KERNEL_VERSION
ENV BUSYBOX_VERSION=$BUSYBOX_VERSION

RUN apt update && \
    apt install -y build-essential libncurses-dev bison flex \
    libssl-dev libelf-dev dwarves bc xz-utils curl file cpio

WORKDIR /builder

# Download and extract kernel & BusyBox sources
RUN curl -L https://cdn.kernel.org/pub/linux/kernel/${KERNEL_CHANNEL}/linux-${KERNEL_VERSION}.tar.xz | tar xJf - && \
    curl -L https://busybox.net/downloads/busybox-${BUSYBOX_VERSION}.tar.bz2 | tar xjf -

# Copy contents of tools/ without the directory itself
COPY ./tools .

ENTRYPOINT ["make"]
