_default:
    just --list

debug_build := "./target/debug/needs"
release_build := "./target/release/needs"

@build:
    cargo build --release &> /dev/null

@build_no_versions:
    cargo build --release --no-default-features &> /dev/null

@bench: build
    hyperfine '{{ release_build }}' '{{ release_build }} --no-version' '{{ release_build }} --quiet' \
        -N --warmup 50 -M 500 -i --export-markdown report.md

@bench_no_versions: build_no_versions
    hyperfine '{{ release_build }}' '{{ release_build }} --quiet' \
        -N --warmup 50 -M 500 -i --export-markdown report.md

@install:
    cargo install --path .
