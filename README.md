# Blockchain Rust Project


This project is my final work for the Rust course at the Polkadot Academy, offered by PUC-PR. It is a blockchain implementation developed in Rust, featuring functionalities for managing transactions, blocks, and nodes, as well as a basic consensus mechanism to ensure network reliability.
## Features

- **Blockchain**: Manage a chain of blocks, each containing transactions.
- **Transactions**: Create and execute transactions between accounts.
- **Consensus**: Simple consensus mechanism to ensure the blockchain is valid.
- **Nodes**: Connect and synchronize with other nodes in the network.
- **REST API**: Interact with the blockchain via a REST API.

## Prerequisites

- Rust (latest stable version)
- Cargo (Rust package manager)
- Rocket (web framework for Rust)
- Tokio (asynchronous runtime for Rust)

## Installation

1. **Clone the repository**:
    ```sh
    git clone https://github.com/giorgeabdala/abdala-chain
    cd blockchain-rust
    ```

2. **Build the project**:
    ```sh
    cargo build
    ```

3. **Run the tests**:
    ```sh
    cargo test
    ```

## Usage

1. **Start the server**:
    ```sh
    cargo run
    ```

2. **Interact with the API**:
    - **Get the blockchain**:
        ```sh
        curl http://localhost:8000/get_chain
        ```
    - **Check if the blockchain is valid**:
        ```sh
        curl http://localhost:8000/is_valid
        ```
    - **Add a transaction**:
        ```sh
        curl -X POST -H "Content-Type: application/json" -d '{"sender": "Alice", "to": "Bob", "amount": 50, "message": ""}' http://localhost:8000/add_transaction
        ```
    - **Connect a new node**:
        ```sh
        curl -X POST -H "Content-Type: application/json" -d '{"nodes": ["http://localhost:8001"]}' http://localhost:8000/connect_node
        ```
    - **Get balance of an address**:
        ```sh
        curl http://localhost:8000/balance?address=Alice
        ```
    - **Get nonce of an address**:
        ```sh
        curl http://localhost:8000/get_nonce?address=Alice
        ```

## Project Structure

- `src/`
    - `main.rs`: Entry point of the application.
    - `runtime.rs`: Contains the `Blockchain` struct and its implementation.
    - `core_client/`
        - `balance.rs`: Manages account balances.
        - `system.rs`: Manages nonces for accounts.
        - `rpc.rs`: Defines the REST API routes and handlers.
    - `domain/`
        - `block.rs`: Defines the `Block` struct.
        - `transaction.rs`: Defines the `Transaction` struct.

## Contributing

1. Fork the repository.
2. Create a new branch (`git checkout -b feature-branch`).
3. Make your changes.
4. Commit your changes (`git commit -am 'Add new feature'`).
5. Push to the branch (`git push origin feature-branch`).
6. Create a new Pull Request.

## License

This project is licensed under the MIT License. See the `LICENSE` file for more details.


Giorge Abdala