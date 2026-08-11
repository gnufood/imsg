# Service Management

This section covers how to install, configure, and control the imsg daemon as a system service managed by your operating system's service manager (such as systemd on Linux or launchd on macOS).

Whether you need to set up the daemon to start automatically on boot, understand how service identity and permissions work, or programmatically control the service through code, the documentation here provides the guidance you need.

---

## Installation & Setup

Start here if you want to get the daemon running as a system service that starts automatically when your system boots.

- **[Installing the Daemon as a System Service](install-daemon-as-service.md)** — A practical walkthrough for registering the imsg daemon with your OS service manager (systemd, launchd, etc.) so it runs as a persistent background service. Covers the registration process, enabling the service, and verifying it's running.

---

## Understanding Service Behavior

Learn why the service behaves the way it does, particularly around user identity and privilege handling.

- **[System Service Identity Resolution](system-service-identity-resolution.md)** — An explanation of why system services run as the invoking user rather than root, and how sudo detection works in this context. Helps you understand the security model and permission boundaries when the daemon runs as a managed service.

---

## Programmatic Control

Reference material for developers who need to control the service from code or scripts.

- **[Service Management API Reference](service-management-api.md)** — Complete reference for the public API that allows you to install, uninstall, start, stop, and query the status of the daemon as an OS service. Includes function signatures, parameters, and usage patterns for integration into your own applications.