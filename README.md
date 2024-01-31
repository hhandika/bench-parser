# bench-parser

Parse GNU time benchmark files into CSV format. See the SHELL scripts in the [segul-bench](https://github.com/hhandika/segul-bench) as examples.

## Installation

You need to have Rust installed. See [Rust installation guide](https://www.rust-lang.org/tools/install) for more information.

```bash
cargo install --git https://github.com/hhandika/bench-parser.git
```

```bash
bench-parser -i <input> -o <output>
```

By default, it parses benchmark with five replicates. To change the number of replicates, use the `-s` flag.

```bash
bench-parser -i <input> -o <output> -s <number of replicates>
```
