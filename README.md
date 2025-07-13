# Mycelia Mesh

[![Crates.io](https://img.shields.io/crates/v/mycelia-mesh)](#) [![Docs](https://img.shields.io/badge/docs-latest-blue)](#) [![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

**A peer-to-peer Ollama LLM fabric that self-organizes across your devices**—no cloud, no vendor lock-in, true edge intelligence.

---

## 🚀 Features

- **Local-first LLM**  
  Runs any Ollama-compatible model entirely on your network—data never leaves your devices.

- **Organic Scaling**  
  Multiple machines pool CPU/GPU cycles automatically via mDNS + libp2p gossip.

- **Context Gravity**  
  Shards of your knowledge move to the nodes that need them most for sub-second response.

- **Zero Single Point of Failure**  
  Fully decentralized mesh; nodes join or drop without disrupting the network.

- **Drop-in Ollama API**  
  Compatible with existing Ollama clients—just point them at `http://localhost:11434`.

---

## 📦 Quick Start

### 1. Clone & Build  
```bash
git clone https://github.com/Darkstar420/mycelia-mesh.git
cd mycelia-mesh
cargo build --release -p mycelia-node
