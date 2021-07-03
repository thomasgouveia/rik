# This Dockerfile aims to run the project on a distribution which is not
# supported by Runc, Skopeo and Umoci like macOS.
# This is only for development purposes.
FROM alpine:latest
RUN apk add --no-cache runc skopeo umoci rust cargo protoc

WORKDIR /app

COPY ./src ./src
COPY ./Cargo.* ./

RUN cargo build

ENTRYPOINT ["./target/debug/riklet", "--master-ip", "172.20.0.2:4995"]