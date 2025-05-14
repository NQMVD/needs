release_build := "./target/release/needs"

@_default:
    just --list

@testlog:
    clear
    cargo r -- eza -vvvv
    
@test_cases:
    clear
    cd ./needsfiles/always_present && cargo r -- -vvvv
    hr
    cd ./needsfiles/builtins && cargo r -- -vvvv
    hr
    cd ./needsfiles/collection && cargo r -- -vvvv
    hr
    -cd ./needsfiles/empty && cargo r -- -vvvv
    hr
    cd ./needsfiles/never_present && cargo r -- -vvvv
    hr
    -mkdir ./needsfiles/non_existent
    -cd ./needsfiles/non_existent && cargo r -- -vvvv
    
@test_all:
    cargo clippy
    cargo test
    cd ./needsfiles/always_present && cargo r
    cd ./needsfiles/builtins && cargo r
    cd ./needsfiles/collection && cargo r
    -cd ./needsfiles/empty && cargo r
    cd ./needsfiles/never_present && cargo r
    -mkdir ./needsfiles/non_existent
    -cd ./needsfiles/non_existent && cargo r

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

@install_no_versions:
    cargo install --path . --no-default-features

@install_from_cratesio:
    cargo install needs

@freeze_all: install freeze_latest freeze_no_versions freeze_help freeze_log
    echo "All images have been generated."

@freeze_help:
    freeze -c full -x "needs --help" -o "images/needs_help.png"

@freeze_latest:
    freeze -c full -x "needs" -o "images/needs_latest.png"

@freeze_no_versions:
    freeze -c full -x "needs --no-versions" -o "images/needs_no_versions.png"

@freeze_log:
    freeze -c full -x "needs -vvv" -o "images/needs_log.png"
