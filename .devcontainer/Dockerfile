FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# Include lld linker to improve build times either by using environment variable
# RUSTFLAGS="-C link-arg=-fuse-ld=lld" or with Cargo's configuration file (i.e see .cargo/config.toml).
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends clang lld sqlite3 pkg-config libasound2-dev \
    && apt-get autoremove -y && apt-get clean -y

RUN rustup update stable
RUN rustup default stable
RUN rustup component add rustfmt rust-docs clippy

RUN cargo install sqlx-cli --no-default-features --features native-tls,sqlite

USER vscode

RUN sudo chown -R $(whoami) /usr/local/* \
    && sudo chmod -R g+rwx /usr/local/

RUN git config --global --add safe.directory /workspaces/lokai
