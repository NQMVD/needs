release_build := "./target/release/needs"

@_default:
    just --list
    needs gum freeze hr termframe resvg agg

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
    cd ./needsfiles/always_present && ../../target/release/needs -vvvv
    hr
    cd ./needsfiles/builtins && ../../target/release/needs -vvvv
    hr
    cd ./needsfiles/collection && ../../target/release/needs -vvvv
    hr
    -cd ./needsfiles/empty && ../../target/release/needs -vvvv
    hr
    cd ./needsfiles/never_present && ../../target/release/needs -vvvv
    hr
    -mkdir ./needsfiles/non_existent
    -cd ./needsfiles/non_existent && ../../target/release/needs -vvvv

test: build
    cargo clippy
    cargo test
    ./target/release/needs
    ./target/release/needs grep -q
    -./target/release/needs ADFBHYNIL -q
    ./target/release/needs -n
    cd ./needsfiles/always_present && ../../target/release/needs
    cd ./needsfiles/builtins && ../../target/release/needs
    cd ./needsfiles/collection && ../../target/release/needs
    -cd ./needsfiles/empty && ../../target/release/needs
    cd ./needsfiles/never_present && ../../target/release/needs
    @-mkdir ./needsfiles/non_existent
    -cd ./needsfiles/non_existent && ../../target/release/needs -q
    @hr
    @gum log -l "info" "All tests passed."

test-no-versions: build-no-versions
    cargo clippy --no-default-features
    cargo test --no-default-features
    ./target/release/needs
    ./target/release/needs grep -q
    -./target/release/needs ADFBHYNIL -q
    ./target/release/needs -n
    cd ./needsfiles/always_present && ../../target/release/needs
    cd ./needsfiles/builtins && ../../target/release/needs
    cd ./needsfiles/collection && ../../target/release/needs
    -cd ./needsfiles/empty && ../../target/release/needs
    cd ./needsfiles/never_present && ../../target/release/needs
    @-mkdir ./needsfiles/non_existent
    -cd ./needsfiles/non_existent && ../../target/release/needs -q
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

@run-from-pkgx:
    pkgx -Q needs
    pkgx needs --version
    pkgx needs

@freeze-all: freeze-latest freeze-no-versions freeze-help freeze-log
    gum log -l "info" "All images have been generated."

# @freeze cmd path:
#     gum spin --show-output --show-error --title="Freezing: '{{ cmd }}' to {{ path }}" -- freeze -c full -w 95 -x "{{ cmd }}" -o "{{ path }}"

# @freeze cmd path:
#     gum spin --show-output --show-error --title="Freezing:  '{{ cmd }}' to {{ path }}" -- \
#         termframe \
#         -W 95 \
#         --window-style "new-macos" \
#         --theme "vepser" \
#         --font-family "JetBrains Mono" \
#         --embed-fonts true \
#         -o "{{ path }}.svg" -- {{ cmd }}
#     gum spin --show-output --show-error --title="Rendering: '{{ cmd }}' to {{ path }}" -- \
#         resvg \
#         --zoom 4 \
#         --dpi 144 \
#         --use-font-file "/Users/noah/Library/Fonts/BerkeleyMonoVariable-Regular.ttf
#         --font-family "Berkeley Mono Variable" \
#         --monospace-family "Berkeley Mono Variable" \
#         --font-size 16 \
#         "{{ path }}.svg" "{{ path }}.png"

@freeze cmd path:
    gum spin --show-output --show-error --title="Freezing:  '{{ cmd }}' to {{ path }}" -- \
        termframe \
        -W 95 \
        --window-style "new-macos" \
        --theme "vepser" \
        -o "{{ path }}.svg" -- {{ cmd }}
    gum spin --show-output --show-error --title="Rendering: '{{ cmd }}' to {{ path }}" -- \
        resvg \
        --zoom 4 \
        --dpi 144 \
        --font-size 16 \
        "{{ path }}.svg" "{{ path }}.png"

@freeze-help:
    just freeze "needs --help" "images/needs_help"

@freeze-latest:
    just freeze needs "images/needs_latest"

@freeze-no-versions:
    just freeze "needs --no-versions" "images/needs_no_versions"

@freeze-log:
    just freeze "needs -vvv" "images/needs_log"
