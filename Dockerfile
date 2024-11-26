FROM rust:latest

# Crea una directory per i dati persistenti
RUN mkdir -p /data

COPY ./ ./

RUN cargo build --release

# Usa un volume per salvare i bindings
VOLUME ["/data"]

CMD ["./target/release/rustbusters-support"]