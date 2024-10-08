_default:
    just --list

test:
    #!/usr/bin/env bash
    echo ">>> test: type, has, needs"
    echo "> test helix"
    type -a helix
    has helix
    needs helix
    echo "> test zellij"
    type -a zellij
    has zellij
    needs zellij
    echo "> test rust"
    type -a rust
    has rust
    needs rust
    echo "> test cargo"
    type -a cargo
    has cargo
    needs cargo
    echo "> test lua"
    type -a lua
    has lua
    needs lua
