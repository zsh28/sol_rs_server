# SOLANA FELLOWSHIP

## Overview
This project is a Rust-based HTTP server designed for high performance and scalability. It provides a robust foundation for building web applications and APIs.

## Features
- **Fast and Efficient**: Built with Rust, ensuring high performance and low memory usage.
- **Modular Design**: Organized into multiple modules for better maintainability.
- **OpenAPI Integration**: Includes OpenAPI support for API documentation.
- **JSON Extraction**: Provides utilities for extracting and processing JSON data.

## Project Structure
```
├── Cargo.lock
├── Cargo.toml
├── Dockerfile
├── jest.config.js
├── package.json
├── src/
│   ├── json_extractor.rs
│   ├── main.rs
│   ├── openapi.rs
│   ├── routes.rs
├── tests/
│   ├── api_tests.js
```

## Getting Started

### Prerequisites
- Rust (latest stable version)
- Node.js (for running tests)
- Docker (optional, for containerization)

### Installation
1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd sol_rs_server
   ```
2. Build the project:
   ```bash
   cargo build
   ```
3. Run the server:
   ```bash
   cargo run
   ```

### Running Tests
To run the API tests:
```bash
npm install
npm test
```

## API Documentation
The server includes OpenAPI support for API documentation. You can access the documentation at `/` endpoint when the server is running.

## API Routes

### `/submit`
- **Method**: POST
- **Description**: Accepts a message payload and echoes it back with a status of "Received".

### `/balance/{address}`
- **Method**: GET
- **Description**: Fetches the balance of a given Solana address in lamports and SOL.

### `/keypair`
- **Method**: POST
- **Description**: Generates a new Solana keypair and returns the public key and secret key.

### `/token/create`
- **Method**: POST
- **Description**: Creates a new token mint on the Solana blockchain. Requires mint address, mint authority, and decimals.
