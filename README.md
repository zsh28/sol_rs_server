# SOLANA FELLOWSHIP

## Overview
This project is a Rust-based HTTP server optimized for performance and scalability. It serves as a reliable platform for developing web applications and APIs, leveraging Rust's efficiency and safety features. The server is designed to interact seamlessly with the Solana blockchain, enabling functionalities such as token creation, balance retrieval, and secure transactions.

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

### `/token/mint`
- **Method**: POST
- **Description**: Mints tokens to a specified destination address. Requires mint address, destination address, authority, and amount.

### `/message/sign`
- **Method**: POST
- **Description**: Signs a message using a provided secret key.

### `/message/verify`
- **Method**: POST
- **Description**: Verifies the validity of a signed message using the provided signature and public key.

### `/send-sol`
- **Method**: POST
- **Description**: Transfers SOL from one address to another. Requires sender address, recipient address, and amount in lamports.

### `/send-token`
- **Method**: POST
- **Description**: Transfers tokens from one address to another. Requires destination address, mint address, owner address, and amount.
