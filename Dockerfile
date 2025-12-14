FROM rust:slim-buster AS builder

WORKDIR /prod

# Fix repository URLs for archived Buster
RUN echo "deb http://archive.debian.org/debian/ buster main contrib non-free" > /etc/apt/sources.list && \
    echo "deb http://archive.debian.org/debian-security/ buster/updates main contrib non-free" >> /etc/apt/sources.list

# following needed for `openssl-sys v0.9.98`:
RUN apt-get update -y
RUN apt-get install -y pkg-config
RUN apt-get install -y libssl-dev

COPY . .
RUN cargo build --release

FROM fedora:34 AS runner
COPY --from=builder /prod/target/release/fly /bin
COPY --from=builder /prod/assets/ /assets/
COPY --from=builder /prod/shared/data/model/ /model/
RUN ls -R .

CMD ./bin/fly --model-dir ./model --opentelemetry