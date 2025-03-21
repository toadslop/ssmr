# SSMR

A port of the AWS's session manager plugin to Rust.

## Goals

- provide a Rust-based replacement for high performance use cases where golang's GC might be an issue
- expose the ssm plugin's internals as a library so AWS SDKs can directly manage ssm sessions without relying on the ssm plugin