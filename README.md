# eBUS

[![CI](https://github.com/torkleyy/ebus/actions/workflows/ci.yml/badge.svg)](https://github.com/torkleyy/ebus/actions/workflows/ci.yml)
[![API docs](https://img.shields.io/badge/API%20docs-blue)](https://torkleyy.github.io/ebus/energy_bus/index.html)
![License](https://img.shields.io/github/license/torkleyy/ebus)

Software driver for [eBUS] (energy bus) written in Rust.

[eBUS]: https://ebus-wiki.org/lib/exe/fetch.php/ebus/spec_prot_12_v1_3_1_e.pdf

* `no-std`
* few dependencies:
    * [`log`] (optional)

[`heapless`]: https://github.com/japaric/heapless
[`log`]: https://github.com/rust-lang/log

## Features

* [x] Lightweight API allowing for different execution models
* [x] Priority-based collision resolution
* [x] Sending Master-Slave telegram
* [x] Receiving Master-Slave telegram
* [x] Master-Master
* [ ] Sniffing
* [ ] Broadcast

## Integration

See [the integration example](examples/integration.rs).

## License

This software is licensed under Apache-2.0.
