FROM rust:1.92.0-bookworm AS builder

WORKDIR /prod

COPY . .
RUN cargo build --release

FROM debian:bookworm-slim AS runner
COPY --from=builder /prod/target/release/fly /bin
COPY --from=builder /prod/assets/ /assets/
COPY --from=builder /prod/shared/data/model/ /model/
COPY --from=builder /prod/blog/content/posts/ /blog/posts/
RUN ls -R .

CMD ./bin/fly --model-dir ./model --blog-content-dir ./blog/posts --current-year 2026 --selectable-years "2024 2025 2026"
