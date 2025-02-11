_default:
    just --list

test:
    cargo b
    ./target/debug/needs hx nu eza apfelkuchen
    ./target/debug/needs -q hx nu eza
    ./target/debug/needs -q hx nu eza apfelkuchen || echo "failed as expected"
