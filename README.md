# ChatGPT History Explorer

A Rust tool for importing and exploring ChatGPT conversation history from exported zip files.

## Features

- Extract conversations data from ChatGPT export zip files
- Import conversations into one single SQLite database file(assets file as blob)
- Full-text search support using SQLite native search
- TODO: create desktop app to explore your data
- TODO: RAG base semantic search
- TODO: chat with data

## Usage

### Import conversations from zip file

```bash
cargo run --bin importer path/to/chatgpt-export.zip
```

Or specify custom output database:

```bash
cargo run --bin importer path/to/chatgpt-export.zip --output my-conversations.db
```

## Database Schema

The tool creates an optimized SQLite database with three main tables:

- **conversations**: Stores conversation metadata (title, timestamps, model info)
- **messages**: Stores message content and metadata
- **assets**: Stores file content as BLOB data with metadata

Full-text search is enabled on message content for fast searching.

## License

This project is licensed under the MIT License:

```
MIT License

Copyright (c) 2025

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
