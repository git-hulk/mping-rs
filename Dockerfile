# 第一阶段：构建
FROM rust:slim-buster as builder
RUN mkdir -p /usr/src/mping-rs
WORKDIR /usr/src/mping-rs

COPY . /usr/src/mping-rs/
RUN cargo build --release

# 第二阶段：创建最终镜像
FROM debian:buster-slim
COPY --from=builder /usr/src/mping-rs/target/release/mping /usr/local/bin/mping

ENTRYPOINT ["/usr/local/bin/mping"]