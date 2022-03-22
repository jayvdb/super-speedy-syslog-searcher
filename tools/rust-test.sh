#!/usr/bin/env bash
#
# run `cargo-test` in one command with parameters I strongly prefer

set -eu

cd "$(dirname -- "${0}")/.."

export RUST_BACKTRACE=1

# allow user to pass -- to place extra arguments past the prescripted -- delimiter
declare -a args1=()
for a in "${@}"; do
    if [[ "${a}" == "--" ]]; then
        shift
        break
    fi
    args1[${#args1[@]}]=${a}
    shift
done
declare -a args2=()
for a in "${@}"; do
    args2[${#args2[@]}]=${a}
done

set -x

exec cargo test -j1 "${args1[@]}" -- --test-threads=1 "${args2[@]}"
