# SSMR

A port of the AWS's [session manager plugin](https://github.com/aws/session-manager-plugin) to Rust.

This project is a work in progress.

## Goals

- provide a Rust-based replacement for high performance use cases where golang's GC might be an issue
- expose the ssm plugin's internals as a library so AWS SDKs can directly manage ssm sessions without relying on the ssm plugin

## Sub Projects

The design of this port is slightly different than the original plugin. The original plugin is designed to be a standalone binary, so it doesn't expose any of its internals for consumption apart from the binary itself. This project, on the other hand, aims to make not just a port of the application, but also to make the internals available as a library for other SDKs. For that reason, the project is devided into two separate crates, `session-manager-plugin`, which is a standalone binary, and `ssm-lib`, which exports the functions necessary to interact with server-side SSM using SSM's internal protocol.

### session-manager-plugin

The standalone binary. The only code which should be ported here is code concered with parsing command line input and forwarding it to the proper handling functions.

### ssm-lib

The library crate. Most of the code from the original `session-manager-plugin` will be ported here.

## Contributing

Contributions are welcome. To contribute, check the [session-manager-plugin repo](https://github.com/aws/session-manager-plugin), find some code that hasn't been ported yet, and port it over.

Bindings to other languages are also welcome contributions.

THIS IS A TEST