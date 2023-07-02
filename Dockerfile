## Build stage
## Build mimalloc
FROM alpine:3.18 as mimallocbuilder
RUN apk add git build-base cmake linux-headers
RUN cd /; git clone --depth 1 https://github.com/microsoft/mimalloc; cd mimalloc; mkdir build; cd build; cmake ..; make -j$(nproc); make install

## Build pdf-converter binary
FROM rust:1.70-alpine3.18 AS builder

WORKDIR /usr/src
RUN USER=root cargo new pdf-converter
COPY Cargo.toml Cargo.lock pdf.proto build.rs /usr/src/pdf-converter/
WORKDIR /usr/src/pdf-converter/
RUN apk add --no-cache musl-dev protoc && rustup target add x86_64-unknown-linux-musl
RUN update-ca-certificates
RUN cargo build --target x86_64-unknown-linux-musl --release
COPY src /usr/src/pdf-converter/src/
RUN touch /usr/src/pdf-converter/src/main.rs 
RUN cargo build --target x86_64-unknown-linux-musl --release
RUN strip -s /usr/src/pdf-converter/target/x86_64-unknown-linux-musl/release/pdf-converter

## Put together final image
FROM asciidoctor/docker-asciidoctor:1 AS runtime
COPY --from=mimallocbuilder /mimalloc/build/*.so.* /lib/
RUN ln -s /lib/libmimalloc.so.2.1 /lib/libmimalloc.so
ENV LD_PRELOAD=/lib/libmimalloc.so
ENV MIMALLOC_LARGE_OS_PAGES=1
COPY --from=builder /usr/src/pdf-converter/target/x86_64-unknown-linux-musl/release/pdf-converter .
EXPOSE 3000
USER 65534
CMD ["./pdf-converter"]

