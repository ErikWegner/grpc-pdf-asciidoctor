FROM asciidoctor/docker-asciidoctor

RUN apk add libc-dev gcc protoc
RUN /bin/bash -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"

