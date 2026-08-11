# CLI Application Documentation

Welcome to the CLI Application documentation. This section covers the `imsg` command-line tool, which provides interface for managing messages, contacts, and configuration across your distributed messaging system.

Whether you're setting up the CLI for the first time, learning how command routing works, or looking up specific commands, the pages here guide you through understanding and using the tool effectively.

---

## Start Here: Getting Started

If you're new to the CLI or need a practical introduction, begin with the walkthrough. It walks you through the complete initial setup process—from installation and device pairing through basic everyday operations—so you can start using the tool confidently.

- **[Getting Started Walkthrough](cli-getting-started.md)** — Step-by-step guide covering initial setup, pairing your device, configuring preferences, and performing basic message operations like listing, sending, and managing messages.

---

## Understanding How It Works

These pages explain the concepts behind the CLI's behavior. Use them when you want to understand the architecture, the reasoning behind certain design choices, or how the tool makes decisions about where to route commands.

- **[Opt-in Dispatch Architecture](cli-opt-in-dispatch.md)** — Explains how the CLI decides whether to access a live device or work with the local store based on sync state. This helps you understand why commands behave differently depending on whether your data is fully synchronized.

---

## Command Reference

When you need precise details about available commands, their options, and expected behavior, consult the reference. This is the place to look up specific subcommands and their usage.

- **[CLI Commands Reference](cli-commands-reference.md)** — Complete reference for all `imsg` subcommands, including message operations (list, get, send, delete, threads, folders, sync), contacts management, configuration, daemon operations, and shell completions.

---

## Choosing Where to Start

| If you want to... | Start with |
|-------------------|------------|
| Set up the CLI and try it out | Getting Started Walkthrough |
| Understand command routing logic | Opt-in Dispatch Architecture |
| Look up a specific command or option | CLI Commands Reference |