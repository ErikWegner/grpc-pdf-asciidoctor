# PDF-Converter

This container has a service running that accepts an
[asciidoctor pdf](https://docs.asciidoctor.org/pdf-converter/latest/) file
and generates a PDF.

The service provides a gRPC interface.

## Run dev container

```bash
docker compose -f .devcontainer/docker-compose.yml up -d
docker compose -f .devcontainer/docker-compose.yml exec app /bin/bash
source ~/.cargo/env
```

