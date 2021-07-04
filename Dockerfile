# This Dockerfile aims to run the project on a distribution which is not
# supported by Runc, Skopeo and Umoci like macOS.
# This is only for development purposes.
FROM alpine:latest
RUN apk add --no-cache rust cargo protoc

WORKDIR /app

COPY scheduler/src/ ./scheduler/src/
COPY scheduler/Cargo.* ./scheduler/
COPY node_metrics/src/ ./node_metrics/src/
COPY node_metrics/Cargo.* ./node_metrics/
COPY proto/src/ ./proto/src/
COPY proto/Cargo.* ./proto/
COPY proto/build.rs ./proto/

RUN cd scheduler && cargo build --release
RUN mv ./scheduler/target/release/rik-scheduler /app/rik-scheduler
RUN rm -rf /app/scheduler /app/node_metrics /app/proto

ENTRYPOINT ["/app/rik-scheduler", "-v"]
