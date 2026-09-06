<div align="center">

[<img src="https://raw.githubusercontent.com/apxllo123/duxo/main/resources/icon.png?v=12" width="144" alt="Duxo icon"/>](https://github.com/apxllo123/duxo)

#  ✦ Duxo ✦

<strong>Universal Game Analysis & Adaptation</strong>

<p>Duxo analyzes a game's files, executables, dependencies, runtime requirements, graphics stack, and protection signals before an adaptation plan is generated.</p>

[![ Build](https://img.shields.io/github/actions/workflow/status/apxllo123/duxo/.github/workflows/%EF%A3%BF.yml?label=%EF%A3%BF%20Build&style=for-the-badge&color=4c8bf5)](https://github.com/apxllo123/duxo/actions/workflows/%EF%A3%BF.yml)
[![Release](https://img.shields.io/github/v/release/apxllo123/duxo?display_name=tag&style=for-the-badge&color=7c5cff)](https://github.com/apxllo123/duxo/releases)
[![License](https://img.shields.io/github/license/apxllo123/duxo?style=for-the-badge&color=2ea043)](LICENSE)

</div>

---

## Overview

Duxo is an experimental game analysis and adaptation platform built around a simple rule: **understand the game first, then decide what can be adapted**.

Instead of immediately attempting to transform a game, Duxo builds a structured model of the game directory and its Windows binaries. That model can then be consumed by the Adapter to produce an explicit per-game plan, including dependencies that still need to be resolved and capabilities that are not yet implemented.

The repository currently contains the foundation of that pipeline: recursive analysis, PE inspection, dependency modeling, protection detection, capability profiling, adaptation-plan generation, and a desktop shell that connects the pieces together.

## ✨ What Duxo Does Today

### 🔎 Analyzer

The Rust Analyzer builds a machine-readable manifest without modifying the source game files. It currently supports:

- Recursive game-directory inventory
- File metadata and SHA-256 hashing
- Executable candidate detection
- PE format and architecture detection
- PE headers and section inspection
- PE import inspection
- Binary dependency relationships
- Protection-signal detection, including packer/protector and anti-cheat indicators
- Per-game capability profiling

### 🧩 Adapter

The Rust Adapter consumes the Analyzer's profile and produces a structured adaptation plan.

Plans can describe:

- Graphics requirements and backend needs
- Windows API families
- Runtime requirements
- Dependency-resolution work
- Unresolved imports
- Protection information
- Capabilities that are currently blocked because their executor is not implemented yet

Duxo intentionally reports unfinished conversion work as **blocked** instead of presenting it as supported functionality.

### 🖥️ Desktop

The desktop application provides the user-facing layer for the analysis pipeline.

Current desktop functionality includes:

- Native game-folder selection
- Analyzer execution from the UI
- Adapter-plan execution from the UI
- Controlled Electron preload communication
- Native packaging for macOS ARM64 and Windows

## 🏗️ Architecture

```text
                                Duxo Desktop
                           Electron + TypeScript
                                     │
                              Controlled Bridge
                                     │
                           ┌─────────┴─────────┐
                           │                   │
                    Akron Analyzer       Akron Adapter
                         Rust                  Rust
                           │                   │
                           └─────────┬─────────┘
                                     │
                              Game directory
                                     │
                        ┌────────────┴────────────┐
                        │                         │
                     Manifest                GameProfile
                        │                         │
                        └────────────┬────────────┘
                                     │
                              Adaptation Plan
