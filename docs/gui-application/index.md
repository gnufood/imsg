# GUI Application

This section covers the GUI application built with Tauri, which serves as the primary user interface for the system. The documentation here explains how the application initializes, exposes its functionality through commands, and handles data flow between the frontend and backend components.

## Application Startup & Initialization

These pages cover the startup sequence and how the GUI manages its relationship with the daemon process.

- **[Startup Gate](startup-gate.md)** — Explains the initialization state machine that orchestrates app startup, including config validation, store opening, and synchronization. Start here to understand the phases the application goes through from launch to readiness.

- **[Daemon Self-Provisioning](daemon-self-provisioning.md)** — Describes how the GUI spawns itself as a headless daemon when no daemon is running, and the launch classification mechanism that determines startup behavior. Relevant if you need to understand how the daemon lifecycle is managed.

## Command API Reference

- **[Command Reference](command-reference.md)** — Complete reference of all Tauri commands exposed to the frontend, organized by domain: reads, writes, config, daemon, contacts, and discovery. Use this when you need to understand what operations the frontend can invoke and their parameters.

## Data Architecture

- **[Data Flow & DTOs](data-flow.md)** — Explains how data moves through the GUI application, covering the dual-store pattern, DTO transformations, and the boundary between local and broker-mediated operations. Refer to this to understand how the frontend manages state and communicates with backend services.