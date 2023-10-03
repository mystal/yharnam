# Yharnam

This is a fork of the original `yharnam` repository which updates the supported
Yarn Spinner version to 2.3, and adds in additional methods and functions.

A Rust implementation of the [Yarn Spinner] runtime. Currently it is only
capable of running pre-compiled Yarn Spinner files. A parser for `.yarn` files
is in progress.

> **NOTE** to run tests you will need to download a copy of
> [`ysc`](https://github.com/YarnSpinnerTool/YarnSpinner-Console), and use it to
> compile each of the test `.yarn` files.

Currently tested on Yarn Spinner
[2.3.0](https://github.com/YarnSpinnerTool/YarnSpinner/releases/tag/v2.3.0).

[Yarn Spinner]: https://yarnspinner.dev/

## Features

### random

Use the `random` feature to enable random operations such as `dice`, `random`
and `random_range`. An additional non-standard function `random_test` is
provided. This takes a threshold between 0-1 and randomly generates a boolean
based on whether a random number is above or below the threshold (see `gen_bool`
from the `rand` crate).

In addition the random generator can be supplied a seed by calling
`vm.seed_random_generator(u64)`. See the random tests for an example.
