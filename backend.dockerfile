FROM rust:trixie AS builder

WORKDIR /sensor-backend

COPY . .

RUN cargo build --bin sensor-backend --release

FROM debian:trixie

WORKDIR /sensor-backend

COPY --from=builder /sensor-backend/target/release/sensor-backend .

EXPOSE 3000/udp
EXPOSE 8080/tcp

ENTRYPOINT ["/sensor-backend/sensor-backend"]
