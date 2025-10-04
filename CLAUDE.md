# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

mdict-rs is a web-based dictionary application built in Rust that supports MDX format dictionary files. It provides a web interface for querying dictionary entries and serves static files.

## Common Development Tasks

### Running the Application
```bash
cargo run --bin mdict-rs
```
The application runs on `http://localhost:8181` by default.

### Building
```bash
cargo build
```

### Code Formatting
This project uses rustfmt with specific configuration:
```bash
cargo fmt
```

## Architecture

### Core Components

- **MDX File Processing** (`src/mdict/`): Parses MDX dictionary files into structured data
  - `header.rs`: Parses MDX file headers
  - `keyblock.rs`: Handles key blocks containing dictionary entries
  - `recordblock.rs`: Manages record blocks with definitions
  - `mdx.rs`: Main MDX file parser and coordinator

- **Indexing System** (`src/indexing/`): Converts MDX files to SQLite databases for efficient querying
  - Creates `.db` files alongside MDX files
  - Tables: `MDX_INDEX(text, def)`

- **Web Server** (`src/main.rs`, `src/handlers/`): Axum-based web server
  - `/query`: POST endpoint for dictionary lookups
  - `/lucky`: GET endpoint for random word lookup
  - Static file serving from `resources/static/`

- **Query System** (`src/query/`): Handles dictionary queries across multiple MDX files
  - Searches through SQLite databases in order defined in config

### Configuration

- **Dictionary Files**: Configured in `src/config/mod.rs` in `MDX_FILES` array
- **Static Files**: Located in `resources/static/`
- **MDX Files**: Located in `resources/mdx/` with language subdirectories

### Data Flow

1. Application starts and indexes all configured MDX files to SQLite databases
2. Web server accepts queries via `/query` endpoint
3. Query system searches through databases in configured order
4. Results returned as plain text responses

### Dependencies

- **axum**: Web framework
- **rusqlite**: SQLite database operations
- **nom**: Parser combinators for MDX file parsing
- **tracing**: Logging and observability
- **flate2**: Compression/decompression for MDX files

## File Structure

```
resources/
├── mdx/           # MDX dictionary files
│   ├── en/        # English dictionaries
│   └── zh/        # Chinese dictionaries
└── static/        # Static files (CSS, etc.)

src/
├── config/        # Application configuration
├── handlers/      # HTTP request handlers
├── indexing/      # MDX to SQLite conversion
├── lucky/         # Random word selection
├── mdict/         # MDX file parsing
├── query/         # Dictionary query logic
└── util/          # Utility functions
```

## Important Notes

- The project uses Rust 2024 edition
- MDX files must be placed in `resources/mdx/` with appropriate subdirectories
- CSS files for MDX dictionaries should be placed in `resources/static/`
- Indexing happens automatically on startup, creating `.db` files alongside MDX files
- The application currently supports MDX version 2.0 with encryption levels 0 and 2