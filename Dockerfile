ARG BASE_IMAGE=ekidd/rust-musl-builder:latest

# Our first FROM statement declares the build environment.
FROM ${BASE_IMAGE} AS builder

# Add our source code.
ADD --chown=rust:rust . ./

# Build our application.
RUN cargo build --release

# Now, we need to build our _real_ Docker container, copying in `using-diesel`.
FROM alpine:latest
RUN apk --no-cache add ca-certificates \
    bash
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/profile-service \
    /usr/local/bin/

COPY ./entrypoint.sh /usr/local/bin/
ENTRYPOINT ["entrypoint.sh"]

CMD /usr/local/bin/profile-service
