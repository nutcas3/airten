# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in AirTen, please report it by emailing
security@airten.dev. Please do not open a public issue.

We will respond to your report within 48 hours and work with you to understand
and resolve the issue.

## Security Considerations

### Memory Safety

AirTen is written in Rust, which provides memory safety guarantees. However:

- FFI boundaries require careful handling
- Unsafe code is minimized and audited
- All public APIs validate inputs

### Audio Processing

- No network access in core library
- No file system access in core library
- Deterministic processing (no random behavior)
- Bounded memory usage

### Dependencies

- Regular security audits via `cargo audit`
- Minimal dependency tree
- Only trusted, well-maintained dependencies

## Best Practices

When using AirTen:

1. Validate all external audio input
2. Use appropriate buffer sizes to prevent overflows
3. Handle errors appropriately
4. Keep the library updated to the latest version
