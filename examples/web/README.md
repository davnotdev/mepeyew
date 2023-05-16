# Web Example

How I test this on my system:

```
RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build --target web
python3 -m http.server
```

