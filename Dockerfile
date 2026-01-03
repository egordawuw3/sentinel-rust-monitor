# ЭТАП 1: Builder
FROM rust:latest as builder

WORKDIR /usr/src/app
COPY . .

# Компилируем бинарник
RUN cargo build --release

# ЭТАП 2: Runtime
FROM debian:bookworm-slim

# Ставим SSL (нужно для HTTPS запросов Телеграма и сайтов)
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Копируем только скомпилированный файл
COPY --from=builder /usr/src/app/target/release/sentinel .

# Создаем папку для базы данных (чтобы примонтировать её как Volume)
RUN mkdir -p data

# Запускаем
CMD ["./sentinel"]
