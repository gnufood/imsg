# Broker Client Documentation

The Broker Client provides the interface for your application to communicate with the broker service over Inter-Process Communication (IPC). This documentation covers everything you need to integrate the broker into your application, understand how the communication pipeline works, and reference the public API when writing code.

Whether you're implementing a new integration, troubleshooting communication issues, or looking up specific function signatures, start here to find the right page for your task.

## Using the Broker Client

These pages guide you through practical, end-to-end scenarios for interacting with the broker. Start here when you need to implement specific features or understand the typical flow of broker operations.

- **[Broker Client Workflows](broker-client-workflow.md)** — Step-by-step scenarios covering common tasks: checking whether the broker is available, sending an SMS message, and querying the current session state. Use these pages to learn how the client is intended to be used in production.

## Understanding the Architecture

These pages explain the concepts and design behind the broker client. Start here when you want to understand how the pieces fit together, why certain decisions were made, or how to troubleshoot communication issues.

- **[Broker Client Architecture](broker-client-architecture.md)** — A conceptual overview of the transport layer, how responses are handled, and how error types propagate through the system. This page helps you build a mental model of reliable broker communication.

## API Reference

This page provides detailed, lookup-style documentation for the public interface. Start here when you need to check function signatures, type definitions, or error codes.

- **[Broker Client API Reference](broker-client-api.md)** — Complete reference for public functions, types, and error definitions used to communicate with the broker over IPC. Consult this page when writing or debugging code that calls the broker client.