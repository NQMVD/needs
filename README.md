# needs
> Check if given bin(s) are available in the PATH.

*...multi-threaded of course.*

## Screenshots
> using cli args

![needs](https://github.com/NQMVD/needs/blob/master/screenshot.png?raw=true)

> using needsfile

![needs](https://github.com/NQMVD/needs/blob/master/screenshot_file.png?raw=true)

## Usage
```bash
needs <bin>...
```

*or*

```bash
# returns 0 if all bins are available, 1 otherwise
needs -q <bin>...
```

### help:
![needs_help](https://github.com/NQMVD/needs/blob/master/needs_help.png?raw=true)

## Installation
```bash
cargo install needs
```
or
```bash
cargo binstall -y needs
```

> [!NOTE]
> Target Platforms are UNIX based systems and Windows support is _not_ planned.
> Because `needs` uses the `which` and `xshell` crates, it might run on Windows anyways.

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

To identify long running binaries, you can use the verbose flag to increase the logging level, which when reached DEBUG (-vvv) shows the timings for the individual binaries.
If you also find poorly optimized programs, just create an issue so i can keep track of them, but you could also just notify the developer and tell him to stop using javascript for big terminal tools :)

## speed
As before mentioned, the speed of the program depends on the called binaries if run with version retrieval.
The speed of the program itself is actually quite fast, see the `report.md` for the results of a benchmark.
The benchmark was done with the `hyperfine` program, run on a M4 Macbook Pro.
