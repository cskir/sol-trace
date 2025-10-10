# sol-trace

## Overview

Rust-based wallet tracking system for the Solana blockchain.  
It allows monitoring of wallet activity and selected tokens in real time, while also laying the groundwork for portfolio analytics and trading strategy evaluation.

This system consists of two main components:

1. **Server Application** – a backend service that exposes a gRPC API, manages client requests, and integrates with external services.
2. **Client Application (CLI)** – a command-line interface that allows users to interact with the server through gRPC calls.

The communication between the server and clients is implemented using **gRPC**, supporting both unary RPCs (simple request/response) and server-side streaming.

## Features

When starting the client, we need to specify the wallet and the tokens we want to examine in a config file.

Features:

- Open and close the subscription for the given wallet's live trades. Command: **sub** / **unsub**
- Get the current portfolio with the latest usd prices. Commad: **hold**
- If a transaction is a trade, return its details. For now, it is more of a debug function, and calculates the trade value by the current usd price, not historical price. Command: **tx [hash]**
- historic trade: coming soon
- position history: coming soon
- position metrics: coming soon

## Server Responsibilities

- **gRPC Service**

  - Implements methods for unary queries and streaming queries.
  - Provides subscription lifecycle management.

- **Subscription Management**

  - Tracks active client subscriptions.
  - Bridges between client subscriptions and external data sources.
  - Streams events to subscribed clients.

- **External Integrations**
  - **WebSocket APIs** for event-driven data feeds.
  - **RPC APIs** for synchronous external calls.
  - Applies internal business logic.

## Client Responsibilities (CLI)

- Provides a terminal user interface for:
  - Sending unary queries
  - Initiating streaming queries to receive continuous updates.
  - Managing subscriptions.
  - Shows logs
- Uses gRPC stubs to communicate with the server.
- Outputs received responses and streaming events in real time.

## Run

cargo run --bin server

cargo run --bin client -- --config .\client_config.toml

## License

- MIT
