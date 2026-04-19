Imagine you have a .dll you want to launch separately (maybe it's malicious or just for fun). In Docker.

Luckily, you have .so as well. Or source code.

Then this project is an example of how to connect things using gRPC. Good luck.

## Build

- install rust if you don't have it
- `cargo build`
- binaries land in `target/debug/`
- `dll_inner.exe` wants `example.dll` sitting next to it (same folder). `example_host.exe` wants `dll_outer.dll` next to it.
- gRPC bit listens on `127.0.0.1:50051`

## What's happening

- `example.dll` is a payload.
- `example_host.exe` relies on that, but i renamed the dependency to `dll_outer.dll` to not mess with side-loading
- `dll_outer.dll` launches a gRPC client and delegates all dll calls there (they are hardcoded)
- `dll_inner.exe` is a gRPC host that catches delegate calls to the real `example.dll`

## How to adapt

- swap in your real payload dll name everywhere it says `example.dll`
- figure out what the host actually calls:
    - open a "x64 native tools" / VS dev shell and run `dumpbin /EXPORTS path\to\your.dll`.
    - the `name` column is what you need to mirror as `#[unsafe(no_mangle)] pub extern "C" fn ...` in `dll_outer` and as `unsafe extern "C" { fn ... }` under `#[link(name = "your.dll", kind = "dylib")]` in `dll_inner`.
    - if names look like `?foo@@YA...` that's C++ decoration - use `undname` on those strings or compile the export side as `extern "C"` so you get plain symbols
- wire each export through gRPC: extend `proto/one.proto` with rpcs/messages that match the arguments you want to ship over the wire (doesn't have to match the dll ABI 1:1, but the rust shim has to)
- `dll_outer/src/lib.rs`: add matching exported stubs, call the tonic client inside them the same way `add` does now (`expect_initialized`, block_on, etc.)
- `dll_inner/src/main.rs`: declare the same signatures against the real dll, implement the server trait to call those `unsafe` fns
- `cargo build` and hope. if link fails you typo'd a name or calling convention; if load fails at runtime wrong dll path
