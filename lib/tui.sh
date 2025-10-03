#!/usr/bin/env bash
# Lightweight TUI helpers for interactive project scripts.

: "${__TUI_LOADED:=0}"

_tui_supports_color() {
  [ -t 2 ] || return 1
  have tput || return 1
  tput colors >/dev/null 2>&1 || return 1
}

tui_init() {
  [ "$__TUI_LOADED" = "1" ] && return
  __TUI_LOADED=1

  if _tui_supports_color; then
    TUI_BOLD="$(tput bold)"
    TUI_DIM="$(tput dim)"
    TUI_RESET="$(tput sgr0)"
    TUI_BLUE="$(tput setaf 4)"
    TUI_GREEN="$(tput setaf 2)"
    TUI_YELLOW="$(tput setaf 3)"
    TUI_CYAN="$(tput setaf 6)"
    TUI_RED="$(tput setaf 1)"
  else
    TUI_BOLD=""
    TUI_DIM=""
    TUI_RESET=""
    TUI_BLUE=""
    TUI_GREEN=""
    TUI_YELLOW=""
    TUI_CYAN=""
    TUI_RED=""
  fi

  TUI_BULLET="${TUI_GREEN}•${TUI_RESET}"
}

tui_divider() {
  tui_init
  local width="${1:-56}"
  local line
  line="$(printf '%*s' "$width" '' | tr ' ' '─')"
  printf '%s%s%s\n' "$TUI_BLUE" "$line" "$TUI_RESET" >&2
}

tui_heading() {
  tui_init
  printf '%s%s%s\n' "${TUI_BOLD}${TUI_CYAN}" "$1" "$TUI_RESET" >&2
}

tui_note() {
  tui_init
  printf '%s%s%s\n' "$TUI_DIM" "$1" "$TUI_RESET" >&2
}

tui_label_value() {
  tui_init
  local label="$1"
  local value="$2"
  printf '%s%s:%s %s\n' "$TUI_BOLD" "$label" "$TUI_RESET" "$value" >&2
}

tui_bullet_list() {
  tui_init
  local line
  while IFS= read -r line; do
    [ -n "$line" ] || continue
    printf '  %s %s\n' "$TUI_BULLET" "$line" >&2
  done
}

tui_confirm() {
  tui_init
  local prompt="${1:-Proceed?}"
  local default="${2:-N}"
  local hint default_ans reply=""

  case "$default" in
    [Yy]) hint="[Y/n]"; default_ans="y" ;;
    *)    hint="[y/N]"; default_ans="n" ;;
  esac

  if [ -r /dev/tty ]; then
    printf '%s%s%s %s ' "$TUI_BOLD" "$prompt" "$TUI_RESET" "$hint" >&2
    if ! IFS= read -r reply < /dev/tty; then
      reply=""
    fi
  else
    reply=""
  fi

  [ -n "$reply" ] || reply="$default_ans"
  case "$reply" in
    [Yy]|[Yy][Ee][Ss]) return 0 ;;
    *) return 1 ;;
  esac
}

tui_abort() {
  tui_init
  printf '%s%s%s\n' "$TUI_DIM" "${1:-Aborted.}" "$TUI_RESET" >&2
  exit 1
}
