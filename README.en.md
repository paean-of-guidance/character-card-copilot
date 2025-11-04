# Character Card Copilot

<div align="center">

**AI-Powered Character Card Editor**

Edit and manage TavernCard V2 format character cards with AI assistance

![Tauri](https://img.shields.io/badge/Tauri-FFC131?logo=tauri&logoColor=white)
![Vue 3](https://img.shields.io/badge/Vue.js-4FC08D?logo=vue.js&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?logo=typescript&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)

[简体中文](./README.md) | English

</div>

## 📖 Introduction

Character Card Copilot is a desktop application for creating, editing, and managing AI character cards with TavernCard V2 standard format support.

---

## ✨ Main Features

### 📥 **Load Character Cards**
- Import from PNG images (read PNG metadata)
- Import from JSON files
- Full TavernCard V2 format compatibility

### 💾 **Export Character Cards**
- Export as PNG format (embed metadata)
- Export as JSON format
- Preserve all character data and world book content

### 🤖 **AI-Assisted Editing**
- Generate and optimize character card content with AI
- **Supported fields**: description, personality, scenario, first message, example dialogue, system prompt, post-history instructions
- One-click apply AI-generated content

> **Note**: AI editing currently supports only main character card fields

### 📚 **World Book Editor** (Testing)
- Create, edit, delete world book entries
- Search and filter functionality
- Full TavernCard V2 world book specification support
- Complete extensions field support

> **Status**: 🧪 Still testing, features incomplete

### ⚙️ **Custom API Configuration**
- Support multiple AI service providers: OpenAI, Anthropic, OpenRouter
- Custom API endpoints
- API key and model configuration

---

## 🛠️ Tech Stack

- **Frontend**: Vue 3 + TypeScript + TailwindCSS 4.0
- **Backend**: Tauri + Rust
- **State Management**: Pinia
- **AI Integration**: Multiple API provider support

---

## 📦 Quick Start

### Install Dependencies
```bash
pnpm install
```

### Run Development Environment
```bash
pnpm tauri dev
```

### Build Application
```bash
pnpm tauri build
```

---

## 📄 License

[TBD]

---

<div align="center">

**Powered by ❤️ and ☕**

</div>
