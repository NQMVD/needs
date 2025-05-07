_default:
    just --list

debug_build := "./target/debug/needs"
release_build := "./target/release/needs"

@build:
    cargo build --release &> /dev/null

@build_no_version:
    cargo build --release --no-default-features  &> /dev/null

@bench: build
    hyperfine '{{ release_build }}' '{{ release_build }} --no-version' '{{ release_build }} --quiet' \
        -N --warmup 50 -M 500 -i --export-markdown report.md

@bench_no_version: build_no_version
    hyperfine '{{ release_build }}' '{{ release_build }} --quiet' \
        -N --warmup 50 -M 500 -i --export-markdown report.md

@install:
    cargo install --path .
