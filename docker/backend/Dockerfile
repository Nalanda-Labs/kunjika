#copied from https://github.com/emk/rust-musl-builder/blob/main/examples/using-diesel/Dockerfile

ARG BASE_IMAGE=ekidd/rust-musl-builder:latest

# Our first FROM statement declares the build environment.
FROM ${BASE_IMAGE} AS builder

# Add our source code.
ADD --chown=rust:rust ./rbackend/ ./

# Build our application.
RUN cargo build --release

# Now, we need to build our _real_ Docker container, copying in `using-diesel`.
FROM alpine:latest
RUN apk --no-cache add ca-certificates bash
COPY docker/backend/docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/rbackend \
    /usr/local/bin/
CMD ["/bin/bash", "/docker-entrypoint.sh"]