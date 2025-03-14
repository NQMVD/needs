TODO:
- use clap derive
- multi-threading
    - use rayon to iterate in parallel
    - mostly for working with bins installed via nix
    - at some point, add a check using whichs path if the bin is installed via nix to warn the user
- optimize running the command with --version somehow
    - use xshell lib? prolly not, to much around it
    - just use std command