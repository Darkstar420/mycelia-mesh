## 🚀 Project Overview

**Mycelia Mesh** is a **peer-to-peer LLM micro-service fabric** built on top of Ollama that lets you run large language models entirely on your own devices—no cloud, no vendor lock-in, and zero single point of failure.

1. **Local-First LLMs**

   * Each node runs an Ollama-compatible model (e.g. LLaMA 3:8B) and exposes the same REST API as Ollama.
2. **Decentralized Mesh**

   * Nodes discover each other via mDNS + libp2p, gossip peer lists, and maintain an in-memory vector index.
3. **Round-Robin Dispatch**

   * The HTTP shim in `mycelia-node` fans out `/api/generate` and `/api/embeddings` calls across your mesh peers, streaming tokens back to the client.
4. **Resilient Shard Management**

   * Shards of embeddings and fine-tuning adapters move to—and rebalance among—active peers when nodes join or exit.
5. **Drop-In Ollama Replacement**

   * Any Ollama-compatible tool (curl, LangChain, AutoGPT) works unchanged—simply point to `http://localhost:11434`.

We’ve now delivered:

* **Peer discovery** (mDNS)
* **REST shim** (Axum server + JSON-lines streaming)
* **Shard rebalancing** on exit

Next up: phase-2 features like latency-aware routing, federated fine-tuning, BLE side-channels, plus a CI/CD workflow.

---

## 📖 New Thorough README.md

````markdown
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
````

---

## 📦 Quick Start

### Prerequisites

* Rust 1.77+
* Ollama CLI 0.1.34+
* Git

### 1. Clone & Build

```bash
git clone https://github.com/Darkstar420/mycelia-mesh.git
cd mycelia-mesh
cargo build --release -p mycelia-node
```

### 2. Pull a Model

```bash
ollama pull llama3:8b
```

### 3. Run Mesh Nodes

Open two terminals:

```bash
# Terminal 1
./target/release/mycelia-node --model llama3:8b

# Terminal 2
./target/release/mycelia-node --model llama3:8b
```

### 4. Query the Mesh

```bash
curl -s http://localhost:11434/api/generate \
  -H "Content-Type: application/json" \
  -d '{"model":"llama3:8b","prompt":"Explain Mycelia Mesh in 30 words"}'
```

---

## 🛠️ Development

* **Format:** `cargo fmt`
* **Lint:** `cargo clippy --all-targets --all-features -- -D warnings`
* **Test:** `cargo test`

### Running in VS Code

1. Install **rust-analyzer** extension.
2. Open folder in VS Code.
3. Use the integrated terminal to run the above commands.

---

## 🌱 Roadmap

* **Phase 2 Features**

  * Latency-aware peer selection
  * Federated fine-tuning with LoRA + DP
  * BLE side-channel for mobile cores
  * Built-in compliance & audit “Guardian”
  * GitHub Actions CI for auto-build & test

* **Phase 3**

  * GUI dashboard for mesh topology
  * Plugin system for custom agents
  * Production hardened Kubernetes operator

See the **Projects** and **Issues** tabs for details.

---

## 🤝 Contributing

1. Fork the repo
2. Create a branch: `git checkout -b feat/your-idea`
3. Code & test:

   ```bash
   cargo fmt
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test
   ```
4. Push & open a PR against `main`
5. Ensure CI passes; add tests for new behavior.

---

## 📄 License

Apache License 2.0 © 2025 Darkstar420

---

*Mycelia Mesh* — your on-prem, peer-to-peer answer to cloud LLMs.

```
