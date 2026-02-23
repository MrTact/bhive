# ADR 001: Move Off OpenCode

## What

Build a custom Rust headless service from scratch instead of forking/extending OpenCode.

## Why

- Reduced risk of possible memory issues as we scale to hundreds or even thousands of concurrent workers
- Green field skill architecture lets us bypass OpenCode's baked-in tools, allowing for better user customization
- Developer's personal preference for using Rust over Typescript
- Headless architecture to separate the service from any specific UI
