# max7800x-hal
[![Crates.io Version](https://img.shields.io/crates/v/max7800x-hal)](https://crates.io/crates/max7800x-hal)
[![docs.rs](https://img.shields.io/docsrs/max7800x-hal)](https://docs.rs/max7800x-hal)

This is an [Embedded HAL] (Hardware Abstraction Layer) for the MAX78000 microcontroller from Analog Devices.

The HAL is built on top of the [`max78000-pac`] Peripheral Access Crate, which provides low-level access to the MAX78000's registers. The HAL provides a higher-level interface to the MAX78000's peripherals, making it easier to write applications.

[Embedded HAL]: https://crates.io/crates/embedded-hal
[`max78000-pac`]: https://github.com/sigpwny/max78000-pac

## Roadmap
See the [roadmap] to see current implementation progress and future plans.

[roadmap]: https://github.com/sigpwny/max7800x-hal/issues/1

> [!NOTE]  
> This HAL is under active development. As a result, the API is volatile and subject to change. Be sure to refer to the [changelog] for breaking changes.

If you want updates for when new releases are made, you can watch this repository by clicking the "Watch" button at the top of the page.

[changelog]: https://github.com/sigpwny/max7800x-hal/releases

## Getting Started
If you already have an existing Rust project, you can add this crate by running:
```sh
cargo add max7800x-hal
```

Otherwise, we recommend getting started using this [Crate template for the MAX78000FTHR board](https://github.com/sigpwny/max78000fthr-template). If you are not using the MAX78000FTHR board, you can still use the template as a reference for setting up your own project.

```sh
cargo generate --git https://github.com/sigpwny/max78000fthr-template
```

## Documentation
Documentation for this HAL can be built by running:
```sh
cargo doc --open
```

Documentation can also be found on [docs.rs](https://docs.rs/max7800x-hal).

## Contributing
We welcome contributions from the community! If you want to contribute to this project, follow the steps below to get started:
1. Fork the repository to create your own copy.
2. Make your changes, commit them, then push them to your fork.
3. Open a [pull request](https://github.com/sigpwny/max7800x-hal/pulls).
4. Maintainers will review your PR and suggest changes if needed.
5. Get merged!

## Maintainers
- [SIGPwny](https://sigpwny.com) of the University of Illinois Urbana-Champaign

We are happy to invite additional maintainers to this repository, especially those who are involved in the eCTF competition! You can contact us at `hello@sigpwny.com` or via [Discord](https://sigpwny.com/discord) to request becoming a maintainer.

## License
This template is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

Copyright (c) 2025 SIGPwny
