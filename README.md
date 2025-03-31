# Kaserve

A high-performance web server inspired by lighttpd, written in Rust.

## Features

### Core Features
- HTTP/HTTPS request handling (HTTP/1.1 and HTTP/2)
- Static file serving with high efficiency
- Virtual hosting for multiple websites on a single server
- URL rewriting and redirection
- Modular architecture with plugin system

### Advanced Features
- FastCGI/SCGI/CGI support
- Compression (gzip/deflate/brotli)
- Security features (TLS/SSL, authentication)
- Load balancing
- Monitoring and logging

## Architecture

Kaserve is built with an event-driven architecture using Tokio for asynchronous operations:

- **Core**: Server lifecycle management, configuration system, event loop
- **Network**: Connection management, HTTP parsing, TLS implementation
- **Handlers**: Static file handling, FastCGI connector, reverse proxy
- **Routing**: URL matching, virtual hosts, URL rewriting
- **Plugins**: Plugin management system and API
- **Security**: Authentication and access control
- **Utils**: Logging, metrics, and compression

## Getting Started

### Building from Source

```bash
git clone https://github.com/yourusername/kaserve.git
cd kaserve
cargo build --release
```

### Running the Server

```bash
./target/release/kaserve --config config.toml
```

### Configuration

See the example configuration file in `examples/config.toml` for available options.

## Performance Optimization

Kaserve implements several performance optimizations:
- Zero-copy techniques with the `bytes` crate
- System `sendfile` calls for efficient file transfers
- Connection pooling
- Multi-level caching
- Optimized work queues
- Memory pooling and buffer reuse

## Roadmap

See the design document in `_design/` for the detailed implementation roadmap.

## License

This project is licensed under the MIT License - see the LICENSE file for details.