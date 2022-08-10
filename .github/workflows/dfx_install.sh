#!/bin/bash

######### Change this to your Versions
RUST_VERSION="${RUST_VERSION:-1.59.0}"
DFX_VERSION="${DFX_VERSION:-0.10.1}"
NODE_VERSION="${NODE_VERSION:-16}"
RUN_INTERNET_IDENTITY="${RUN_INTERNET_IDENTITY:-false}"
#########

FONT_RED='\033[1;31m'
FONT_GREEN='\033[1;32m'
FONT_YELLOW='\033[1;33m'
FONT_BLUE='\033[1;34m'
FONT_PURPLE='\033[1;35m'
FONT_CYAN='\033[1;36m'
FONT_RESET='\033[0m'

function send_mesage() {
    local message_color message_type message_text
    message_color="${1}"
    message_type="${2:-UNDEF}"
    message_text="${3}"

    echo -e "${message_color} $(date '+%F %T') [${message_type}] ${message_text} ${FONT_RESET}"
}

function echo_error() {
    local message
    message="${1}"
    send_mesage "${FONT_RED}" 'ERROR' "${message}"
}

function echo_warn() {
    local message
    message="${1}"
    send_mesage "${FONT_YELLOW}" 'WARN ' "${message}"
}

function echo_info() {
    local message
    message="${1}"
    send_mesage "${FONT_CYAN}" 'INFO ' "${message}"
}

function echo_debug() {
    local message
    message="${1}"
    if [ "${DEBUG}" == 'true' ]; then
        send_mesage "${FONT_BLUE}" 'DEBUG' "${message}"
    fi
}

function echo_success() {
    local message
    message="${1}"
    send_mesage "${FONT_GREEN}" ' OK  ' "${message}"
}

########################### START ###########################


######### Outputing current setup
echo_warn " \n \
######################################################\n \
Current setup will be installed : \n \
\t RUST_VERSION: \t ${RUST_VERSION}\n \
\t DFX_VERSION: \t ${DFX_VERSION}\n \
\t NODE_VERSION: \t ${NODE_VERSION}\n \
\t INTERNET_IDENTITY: \t ${RUN_INTERNET_IDENTITY}\n \
######################################################" >&2
#########

DEBIAN_FRONTEND=noninteractive
DEBUG=true
TZ=UTC  

ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

echo_info "Updating system" >&2
apt-get update && apt-get install -y --no-install-recommends \
        curl \
        git \
        ca-certificates \
        build-essential \
        cmake \
        rsync

# Install node
echo_info "Installing Node ${NODE_VERSION}" >&2
curl  --fail -sSf "https://deb.nodesource.com/setup_${NODE_VERSION}.x" | bash && \
    apt-get install -y nodejs \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Install Rust and Cargo in /opt
echo_info "Installing Rust ${RUST_VERSION}" >&2
export RUSTUP_HOME=/opt/rustup
export CARGO_HOME=/opt/cargo
export PATH="$PATH:/opt/cargo/bin"

curl --fail https://sh.rustup.rs -sSf \
    | sh -s -- -y --default-toolchain ${RUST_VERSION}-x86_64-unknown-linux-gnu --no-modify-path && \
    rustup default ${RUST_VERSION}-x86_64-unknown-linux-gnu

export CARGO_HOME=/cargo
export CARGO_TARGET_DIR=/cargo_target
export PATH="$PATH:/cargo/bin"

# Install DFINITY SDK.
echo_info "Installing DFX ${DFX_VERSION}" >&2
DFX_VERSION=${DFX_VERSION} sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"

rustup target add wasm32-unknown-unknown
cargo install ic-cdk-optimizer --version 0.3.1

echo_info "RUN_INTERNET_IDENTITY = ${RUN_INTERNET_IDENTITY}" >&2
[ "${RUN_INTERNET_IDENTITY}" == 'true' ] && git clone https://github.com/olsemeno/internet-identity.git || echo "Skip II clone" >&2

######### Outputing current result
echo_success " \n \
######################################################\n \
Current result : \n \
$(node --version && rustup --version && dfx --version)\n \
\n
You need add /opt/cargo/bin into your PATH var
######################################################" >&2
#########