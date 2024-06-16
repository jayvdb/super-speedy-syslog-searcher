#!/usr/bin/env bash
#
# Run some of the tools to create release files.
#

set -euo pipefail

cd "$(dirname -- "${0}")/.."

export DIROUT=${DIROUT-.}

(
    set -x
    RUSTFLAGS=-g cargo build --profile flamegraph
    ./tools/flamegraphs.sh
)

(
    set -x
    RUSTFLAGS=-g cargo build --profile valgrind
    ./tools/valgrind-callgrind.sh > "${DIROUT}/callgrind.txt"
)
rm -v "${DIROUT}/callgrind.out" "${DIROUT}/callgrind.dot" || true
sed -i -e "s|$(realpath .)|.|g" "${DIROUT}/callgrind.txt"
sed -i -e "s|${HOME}|/home|g" "${DIROUT}/callgrind.txt"

(
    set -x
    ./tools/valgrind-massif.sh > "${DIROUT}/massif.txt"
)
rm -v "${DIROUT}/massif.out" || true
sed -i -e "s|$(realpath .)|.|g" "${DIROUT}/massif.txt"

(
    # XXX: cargo does not respect color settings
    #      see https://github.com/rust-lang/cargo/issues/9012
    export CARGO_TERM_COLOR=never
    set -x
    cargo bench --no-run
    # require gnuplot to be installed
    gnuplot --version
    cargo bench \
        --benches \
        --quiet \
        --color=never \
        --features bench_jetscii,bench_memchr,bench_stringzilla \
            &> "${DIROUT}/cargo-bench.txt"
)