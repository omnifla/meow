test:
    cargo test

check:
    cargo clippy --all-targets --all-features -- -D warnings

fmt:
    cargo fmt

run example:
    cargo run --example {{example}} --features wgpu,winit
