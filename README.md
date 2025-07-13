# Mycelia Mesh

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache_2.0-blue.svg)](LICENSE) [![Docs](https://img.shields.io/badge/docs-latest-blue)](#) [![Rust](https://img.shields.io/badge/rust-1.77+-lightgray)](#)

**Grow your own LLM micro-service fabric**—peer-to-peer, fully local, and Ollama-compatible.

---

## 🎯 Vision

- **Data Sovereignty:** Keep your documents and prompts on devices you control.  
- **Edge Intelligence:** Leverage spare CPU/GPU cycles across laptops, phones, and Pi boards.  
- **High Availability:** A self-healing mesh with no single point of failure.  
- **Seamless Integration:** Works with existing Ollama clients and tooling without modification.

---

## ⚡ Key Features

1. **Drop-in Ollama API**  
   - Launch a local REST shim on `127.0.0.1:11434` that mirrors `/api/generate` and `/api/embeddings`.

2. **mDNS + libp2p Discovery**  
   - Automatic peer finding on your LAN; encrypted gossip protocol, zero config.

3. **Round-Robin Inference Dispatch**  
   - Fan-out prompts across nodes for parallel inference, then stream tokens back in JSON-lines.

4. **Context Gravity & Shard Cache**  
   - Each node caches document embeddings; the mesh routes queries to whichever node holds the freshest context.

5. **Resilient Shard Rebalancing**  
   - On peer drop-out, orphaned shards are immediately redistributed to remaining nodes.

6. **Extensible Rust Core**  
   - `mesh/` crate for gossip, routing, and shard logic; `cmd/mycelia-node/` for the HTTP shim.

---

## 🏗️ Architecture Diagram

```mermaid
flowchart LR
  subgraph Node A
    A1[HTTP Shim: Axum] --> A2[Mesh Lib]
    A2 --> A3[Shard Cache]
  end

  subgraph Node B
    B1[HTTP Shim: Axum] --> B2[Mesh Lib]
    B2 --> B3[Shard Cache]
  end

  A2 <--> B2[libp2p mDNS & Gossip]
