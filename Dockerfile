FROM rustlang/rust:nightly as builder

RUN USER=root cargo new --bin memory-bus-8080
WORKDIR /memory-bus-8080
COPY ./Cargo.lock ./Cargo.toml ./
RUN cargo build --release
RUN cargo build
RUN rm src/*.rs
COPY ./src ./src
RUN rm -f ./target/release/deps/memory_bus_8080*
RUN cargo build --release
RUN cargo test

FROM ubuntu:20.04

RUN apt update && apt install -y libssl-dev

COPY --from=builder /memory-bus-8080/target/release/memory-bus-8080 .
EXPOSE 8080
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8080
ENV ROCKET_ENV=production
ENTRYPOINT ["./memory-bus-8080"]