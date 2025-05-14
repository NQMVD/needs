<div align="center">
  <h1>
    <a href="https://github.com/NQMVD/needs">needs<a/>
  </h1>
  <h4>Check if given bin(s) are available on your system and, optionally, get their version.</h4>
  <i>...multi-threaded of course..</i>
  <h3></h3>
</div>

<div align="center">

![](https://img.shields.io/github/last-commit/NQMVD/needs?&style=for-the-badge&color=b1ffb4&logoColor=D9E0EE&labelColor=292324)  
![](https://img.shields.io/badge/Rust-fe7a15?style=for-the-badge&logo=rust&logoColor=white&logoSize=auto&labelColor=292324)
[![](https://img.shields.io/crates/v/needs.svg?style=for-the-badge&logoColor=white&logoSize=auto&labelColor=292324)](https://crates.io/crates/needs)  
[![](https://img.shields.io/badge/Charm-Gum-FAA5EA?style=for-the-badge&labelColor=292324)](https://github.com/charmbracelet/gum)
[![](https://img.shields.io/badge/Charm-Freeze-8CFEFE?style=for-the-badge&labelColor=292324)](https://github.com/charmbracelet/freeze)
[![](https://img.shields.io/badge/Bacon-FF8080?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCI+PHBhdGggZmlsbD0id2hpdGUiIGQ9Ik0xNiwxNEMxNi4yMSwxMy41IDE2LjM1LDEzIDEzLjUsMTNDMTEuNSwxMyAxMSwxMy4yNSAxMC41LDEzLjVDMTAsMTMuNzUgOS41LDEzLjUgOS41LDEzLjVDOS41LDEzLjUgOSwxMiA4LDEyQzcsNy41IDEwLDYgMTEsNkMxMiw2IDEzLDYuNSAxMyw2LjVDMTMsNi41IDE0LDYgMTUsNkMxNiw2IDE5LDcuNSAxOCwxMkMxNywxMiAxNi41LDEzLjUgMTYuNSwxMy41QzE2LjUsMTMuNSAxNS43OSwxMy41IDE2LDE0TTIwLDEyQzIwLDguNSAxNi41LDQgMTIsNEM3LjUsNCAxLDEyIDEsMTJDMSwxMiA3LjUsMjAgMTIsMjBDMTYuNSwyMCAyMCwxNS41IDIwLDEyTTEyLDZDMTMuNjYsNiAxNSw3LjM0IDE1LDlDMTUsOS4zNSAxNC45NSw5LjY5IDE0Ljg2LDEwQzE0LjUsOS43IDE0LDkuNSAxMyw5LjVDMTEuNSw5LjUgMTEuNSwxMSAxMCwxMUM5LjUsMTEgOS4xNSwxMC44NSA4Ljg2LDEwLjYzQzkuMDgsOS41NSA5LjUsOC41IDEwLjUsNy41QzEwLjUsNy41IDExLDYgMTIsNloiLz48L3N2Zz4=&labelColor=292324)](https://github.com/Canop/bacon)
[![](https://img.shields.io/badge/Just-000000?style=for-the-badge&logo=just&logoColor=white&labelColor=292324)](https://just.systems)

</a>

</div>


### Screenshots
> using cli args

![needs](https://github.com/NQMVD/needs/blob/master/images/screenshot.png?raw=true)

> using needsfile

![needs](https://github.com/NQMVD/needs/blob/master/images/screenshot_file.png?raw=true)

### Showcase

Here are [freeze](https://github.com/charmbracelet/freeze)-generated screenshots of the latest version:

<details>
<summary>Click to expand</summary>

> just `needs`

![needs_latest](https://github.com/NQMVD/needs/blob/master/images/needs_latest.png?raw=true)

> `needs --no-versions` to skip version retrieval

![needs_no_versions](https://github.com/NQMVD/needs/blob/master/images/needs_no_versions.png?raw=true)

> `needs -vvv` to see what's going on

![needs_log](https://github.com/NQMVD/needs/blob/master/images/needs_log.png?raw=true)

This logging output was hand crafted with _(heavy)_ inspiration from [charm/log](https://github.com/charmbracelet/log) building on [env_logger](https://crates.io/crates/env_logger).  
Ironically not because of the env-part of it, but because it feels like the simplest wrapper of std::log and it supports key-value-pairs which were very important to me.  
For now it will just reside inside of needs, but in the future I'm planning on moving it to it's own lib, maybe call it starlog...  

</details>

---

### Usage

![needs_help](https://github.com/NQMVD/needs/blob/master/images/needs_help.png?raw=true)

---

### Installation
```bash
cargo install needs
```
or
```bash
cargo binstall -y needs
```
or
```bash
# to disable version retrieval completely
cargo install needs --no-default-features
```

> [!NOTE]
> Target Platforms are UNIX based systems, Windows support is _not_ planned.
> Because `needs` uses the `which` and `xshell` crates, it might run on Windows anyways though.

---

### Plans
- [ ] timeouts for calling binaries
- [ ] more version matches
  - [ ] dates and major only (e.g. `openjdk 24 2025-03-18`)
  - [ ] dates with no seperator... (`awk version 20200816`👀)
- [ ] read-from-config-files feature (read ~/.cargo/.crates.toml directly for example)
- [ ] pipe-detection to make scripting easier
- [ ] version requirements via semver (e.g. `needs gum>=0.14.*`)
- [ ] more pretty output formats
  - [x] center aligned
  - [ ] side-by-side (in boxes?)
- [ ] more parsable outptut formats
  - [ ] bash
  - [ ] json
  - [ ] toml
  - [ ] lua?

#### other
- [ ] integrate tests and screenshot generation with workflows

---

## Disclaimer & Insights on calling binaries
### potential modifications
> [!IMPORTANT]
> Keep in mind that needs runs the programs you give it.
> (if it's told to retrieve versions that is...)

This has been tested and shown good results, but there's always one that doesn't work with any conventions or just a really old program.  
In such cases the program doesn't return a version that needs can read, but instead goes off to do it's thing and potentionally ends up making modifications to your filesystem (e.g. deleting files for whatever-reason, we don't know yet).

If you happen to run into one, there's basically nothing I can do for you.  
_Still,_ I would like to try or just hear about it so i can inlude it in a list to prevent future incidents.

### potential latency
The program is inspired by the `has` bash program. Therefore I also wanted it to have the version retrieval feature.  
For now that process relies on the individual binaries getting called with the --version flag,  
which can be _extremely_ slow in some cases (the `mintlify` program for example takes almost an **entire second** to respond).  
Thanks to `par_iter` from rayon it's possible to run all commands in parallel tho, which helps at least a little bit.

For the future I want to improve the multithreading part by introducing a **timeout** for the threads.  
Those that take to long to answer will be terminated after the timeout and the affected binaries will only be marked as found, without a version.

But even with those improvements some might only want to check if the program is installed.  
Therefore I not only added a flag `--no-versions` but also made it a cargo feature which can be disabled with `--no-default-features` when installing.

Also planned is a feature that includes a list of known
- long running,
- uncomplying or old
binaries which will _not_ be called to retrieve their version.

To identify long running binaries, you can use the verbose flag to increase the logging level, which when reached TRACE (-vvvv) shows the timings for the individual binaries.  
If you also find poorly optimized programs, just create an issue so i can keep track of them, but you could also just notify the developer and tell him to stop using javascript for big terminal tools :)

## speed
As before mentioned, the speed of the program depends on the called binaries if run with version retrieval.  
The speed of the program itself is actually quite fast, see the `report.md` for the results of a benchmark.  
The benchmark was done with the `hyperfine` program, run on a M4 Macbook Pro.
