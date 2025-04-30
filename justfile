_default:
    just --list

@bench:
    # hyperfine 'needs' -N --warmup 50
    # hyperfine 'needs --no-version' -N --warmup 50
    # hyperfine 'needs --quiet' -N --warmup 50 -i
    hyperfine 'needs' 'needs --no-version' 'needs --quiet' \
        -N --warmup 50 -M 500 -i --export-markdown report.md
