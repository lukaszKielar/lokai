FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# Include lld linker to improve build times either by using environment variable
# RUSTFLAGS="-C link-arg=-fuse-ld=lld" or with Cargo's configuration file (i.e see .cargo/config.toml).
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install clang lld sqlite3 \
    && apt-get autoremove -y && apt-get clean -y

RUN rustup component add rustfmt rust-docs clippy
RUN rustup update stable

# Tried to install latest (0.8.1 at the time of writing this), but it has a bug
RUN cargo install sqlx-cli@0.8.0 --no-default-features --features native-tls,sqlite

USER vscode

RUN sudo chown -R $(whoami) /usr/local/* \
    && sudo chmod -R g+rwx /usr/local/
