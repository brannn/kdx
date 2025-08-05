# Changelog

## [0.4.1] - 2025-08-05

### Added - Phase 2: Scale and Performance Infrastructure
- Concurrent discovery across multiple namespaces with configurable concurrency limits
- Intelligent caching system with TTL support and automatic expiration
- Pagination support with configurable page sizes and result limits
- Memory optimization features including streaming output and lazy conversion
- Progress tracking with real-time indicators for long-running operations
- Cache management commands: stats, clear, warm
- Performance benchmarking command with concurrent and memory testing
- Global performance flags: --limit, --page-size, --show-progress, --concurrency, --stream, --memory-optimized
- Extended caching for all resource types (deployments, configmaps, secrets, CRDs)
- Memory-efficient processing for large clusters

### Enhanced
- All existing commands now support pagination and caching
- Improved performance for large cluster operations
- Enhanced error handling and resilience
- Comprehensive unit test coverage (88 tests, up from 23)

### Technical
- Thread-safe caching using DashMap
- Kubernetes API pagination with continue tokens
- Concurrent task management with JoinSet and semaphores
- Lazy resource conversion for memory efficiency
- Streaming JSON/YAML output for large datasets

## [0.1.1] - 2025-08-03

### Added
- Multi-platform release builds (Linux x86_64, macOS x86_64, macOS ARM64)
- Automated Homebrew formula updates via GitHub Actions
- Enhanced installation instructions in README

### Changed
- Improved release workflow with cross-platform support
- Updated Homebrew formula to support Linux and Intel macOS

## [0.1.0] - 2025-08-03

### Added
- Service discovery and listing
- Pod exploration with filtering
- Service descriptions with relationships
- Service topology analysis
- Graph visualization (DOT and SVG formats)
- Multiple output formats (table, JSON, YAML)
- Comprehensive CLI interface
- Unit test coverage (23 tests)

### Technical
- Built with Rust for performance
- Uses official Kubernetes client libraries
- Professional CLI with clap
- Error handling and validation

## [0.1.0] - 2025-08-03

Initial release
