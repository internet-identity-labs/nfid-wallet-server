#!/bin/bash

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
