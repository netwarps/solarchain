# This is the build stage for Polkadot. Here we create the binary in a temporary image.
FROM docker.io/paritytech/ci-linux:production as builder

RUN mkdir /usr/src/app
WORKDIR /usr/src/app
ADD . ./
RUN cargo build --locked --release

# This is the 2nd stage: a very small image where we copy the Polkadot binary."
FROM docker.io/library/ubuntu:20.04

LABEL maintainer="kk <3168582@qq.com>"

ARG APP=/usr/src/app
COPY --from=builder ${APP}/target/release/solar-node /usr/local/bin/solar-node

RUN useradd -m -u 1000 -U -s /bin/sh -d ${APP} solar && \
	chown -R solar:solar /data && \
	rm -rf /usr/bin /usr/sbin && \
	/usr/local/bin/solar-node --version

USER solar

EXPOSE 30333 9933 9944
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/solar-node"]
