# Examples

Hello, these are `mepeyew`'s examples.

You can run them for desktop platforms using `cargo r --example ...`.

## Notes for WebAssembly

### Running the Examples

You can run these examples for web using `cargo run-wasm --example ...`.
Note that not all examples fully supported.
Please consult the [documentation](docs.rs/mepeyew) for support details.

### Using `mepeyew` on the Web

Currently, initializing webgpu requires async which is currently not supported.
Because of this, we use the `WebGpuInitFromWindow` extension as a workaround.
You can see its use in the examples.

The easiest way to get setup is to create an extra workspace member in your
`Cargo.toml`.

```
[workspace]
members = [
    "run_wasm"
]
```

Then, implement `run_wasm` exactly as shown in this repo.
This is very important as this project depends on my fork of `run_wasm` and
NOT the original crate.
