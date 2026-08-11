# Configuration

This section covers everything you need to know about configuring imsg, from understanding how configuration layers work to reference documentation for all available settings. Whether you're setting up your first device or customizing advanced options, start here to find the right documentation for your task.

## Reference

The complete reference for all configuration fields supported by imsg. Use these pages when you need to look up what options are available, what values they accept, and what each setting controls.

- [Configuration Options](config-options.md) — Complete reference for all configuration fields, including device settings (MAC address, MAP/PBAP channels), hub settings (node key), store settings (database path), and broker settings (timing and security options)

## Explanation

Background information to help you understand how imsg handles configuration. These pages explain the underlying concepts and mechanisms, so you can make informed decisions about how to configure your setup.

- [Layered Configuration](config-layering.md) — Explains how imsg merges configuration from multiple sources with a clear priority order: compiled defaults → /etc/imsg.toml → XDG user config → local imsg.toml → environment variables, with later sources overriding earlier ones

## Walkthrough

Step-by-step practical guides for common configuration tasks. Follow these when you need to accomplish a specific goal and want hands-on guidance through the process.

- [Configuring a Device](configuring-device.md) — Practical walkthrough for setting up a Bluetooth device, covering how to discover the MAC address and RFCOMM channels, then persist them to your user config file