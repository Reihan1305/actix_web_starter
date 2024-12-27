FROM rust:1.82.0

WORKDIR /actix_starter

RUN cargo install sqlx-cli

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Salin folder src dan migrations ke dalam image
COPY ./src ./src
COPY ./migrations ./migrations
COPY .sqlx .sqlx

ENV SQLX_OFFLINE=true

# Hapus file build lama
RUN rm -f ./target/release/basic_crud*

# Bangun proyek Rust
RUN cargo build --release

# Jalankan migrasi jika file migrasi ada
# RUN sqlx migrate run

CMD ["/basic_crud/target/release/rust_crud_basic"]