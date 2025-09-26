release_build := "./target/release/needs"

@_default:
    just --list
    needs gum freeze hr agg

@gif:
    agg demo.cast --font-family "JetBrainsMono Nerd Font Mono" --speed 2 demo.gif

@work:
    just test
    hr
    just bench
    hr
    just freeze-all
    gum log -l "info" "All tasks have been completed."

@test-cases:
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

test:
    cargo clippy
    cargo clippy --no-default-features
    cargo test
    cargo test --no-default-features
    cargo run
    cargo run --no-default-features
    cargo run -- grep -q
    cargo run --no-default-features -- grep -q
    -cargo run -- ADFBHYNIL -q
    -cargo run --no-default-features -- ADFBHYNIL -q
    cargo run -- -n
    -cargo run --no-default-features -- -n
    cd ./needsfiles/always_present && cargo r
    cd ./needsfiles/builtins && cargo r
    cd ./needsfiles/collection && cargo r
    -cd ./needsfiles/empty && cargo r
    cd ./needsfiles/never_present && cargo r
    @-mkdir ./needsfiles/non_existent
    -cd ./needsfiles/non_existent && cargo r
    @hr
    @gum log -l "info" "All tests passed."

@build:
    cargo build --release &> /dev/null

@build-no-versions:
    cargo build --release --no-default-features &> /dev/null

@bench: build
    hyperfine '{{ release_build }}' '{{ release_build }} --no-version' '{{ release_build }} --quiet' \
      -N --warmup 50 -M 500 -i --export-markdown report.md

@bench-no-versions: build-no-versions
    hyperfine '{{ release_build }}' '{{ release_build }} --quiet' \
      -N --warmup 50 -M 500 -i --export-markdown report.md

@install:
    cargo install --path .

@install-no-versions:
    cargo install --path . --no-default-features

@install-from-cratesio:
    cargo install needs

@freeze-all: install freeze-latest freeze-no-versions freeze-help freeze-log
    gum log -l "info" "All images have been generated."

@freeze cmd path:
    gum spin --show-output --show-error --title="Freezing: '{{ cmd }}' to {{ path }}" -- freeze -c full -w 95 -x "{{ cmd }}" -o "{{ path }}"

@freeze-help:
    just freeze "needs --help" "images/needs_help.png"

@freeze-latest:
    just freeze needs "images/needs_latest.png"

@freeze-no-versions:
    just freeze "needs --no-versions" "images/needs_no_versions.png"

@freeze-log:
    just freeze "needs -vvv" "images/needs_log.png"
