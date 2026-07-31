# 🍒 CherryM-Desktop

<p align="center">
  <b>A lightweight, high-performance YouTube Music desktop client with a Midnight Cherry aesthetic.</b>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Tauri-v2-blue?style=for-the-badge&logo=tauri" alt="Tauri v2" />
  <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/React-61DAFB?style=for-the-badge&logo=react&logoColor=black" alt="React" />
</p>

---

## ✨ Overview

**Cherry** is a custom YouTube Music desktop client designed specifically for Linux environments (optimized for Arch Linux). It combines the speed and efficiency of **Tauri v2** and **Rust** with a modern **React** frontend styled in a deep **Midnight Cherry** theme.

Unlike standard web-wrapper clients, Cherry utilizes a local Rust-based proxy backend that interfaces with `yt-dlp` and YouTube's InnerTube API to provide fast, reliable, and CORS-free audio streaming with minimal memory overhead.

---

## 🛠️ Architecture


```text
┌─────────────────────────────────────────────────────────────┐
│                       React Frontend                        │
│                   (Midnight Cherry Theme)                   │
└──────────────────────────────┬──────────────────────────────┘
                               │ Invoke / Local Stream Request
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                 Rust Backend (Tauri v2)                     │
│  ├── InnerTube API (Search)                                 │
│  ├── Local Axum Proxy Server (127.0.0.1:9876)               │
│  └── System yt-dlp Extractor                                │
└─────────────────────────────────────────────────────────────┘

```

* **Search Engine:** Queries YouTube Music's InnerTube endpoints (`WEB_REMIX`) directly to fetch fast metadata, track titles, duration, and high-res thumbnails.
* **Audio Extractor:** Uses `yt-dlp` under the hood to bypass signature ciphers and PO Token restrictions securely.
* **Stream Proxy:** A local Axum HTTP server running on `127.0.0.1:9876` proxies raw audio chunks back to the client with full HTTP `Range` request support.

---

## 🚀 Getting Started

### Prerequisites (Arch Linux)

Ensure you have the required dependencies installed on your system:

```bash
# Core build dependencies and yt-dlp extractor
sudo pacman -S yt-dlp rust nodejs npm webkit2gtk-4.1
```

### Installation & Development

1. **Clone the repository:**
```bash
git clone https://github.com/LuisGFNunes/cherry.git
cd cherry
```


2. **Install frontend dependencies:**
```bash
npm install
```


3. **Run in development mode:**
```bash
npm run tauri dev
```


4. **Build production bundle:**
```bash
npm run tauri build
```



---

## 🗺️ Roadmap

* [x] Search integration via YouTube Music InnerTube API
* [x] High-performance Rust local proxy for CORS-free audio streaming
* [x] Stable extraction engine using `yt-dlp`
* [ ] Native Linux MPRIS integration (media key controls & OS notifications)
* [ ] Queue management (shuffle, repeat, auto-play next)
* [ ] Audio seek bar & volume control UI synchronization
* [ ] Audio stream pre-fetching for zero-gap playback transitions

---

## 📄 License

Distribute under the **MIT License**. See `LICENSE` for more information.
