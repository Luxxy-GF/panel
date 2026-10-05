# Incus native instances

Calagopus extension `xyz.luxxy.incus` for panel 1.2.4. Adds native OS containers and virtual machines with an image selector and interactive terminal. Requires Wings with the Incus native instance backend and extension metadata support.

## Install

Install `xyz_luxxy_incus.c7s.zip` through the panel extension manager on the heavy panel image, then rebuild and restart the panel using its extension workflow. A source installation uses `panel-rs extensions add <archive>` followed by `panel-rs extensions apply`.

Enable the metadata adapter on the corresponding Wings node:

```yaml
runtime:
  backend: incus
  incus:
    panel_extension: true
```

Keep your existing Incus network and storage configuration. Restart Wings after installing the extension. An enabled adapter requires the extension metadata endpoint; errors stop metadata loading rather than creating application containers for OS servers.

Select an Incus node on the normal create-server page. The extension adds an instance type selector and images advertised by that node. OS instances use an extension-owned operating-system template and bypass application installation scripts. VM creation requires the node to advertise VM support, at least 256 MiB memory, and a positive disk limit.

## Ownership and protocol

All panel feature code lives in this extension. The panel's generated extension registry, frontend compatibility link, migration compatibility link, and dependency lockfile are installation artifacts. No panel core source patch is required.

The extension stores instance metadata in `servers.xyz_luxxy_incus_instance` and includes `incus_instance` in admin and client server responses through API schema extensions. Native creation uses a model creation hook inside the standard server transaction, so the metadata exists before Wings receives the deployment request.

Wings retrieves metadata from authenticated `POST /api/remote/incus/servers` for startup, individual server fetches and configuration updates. Requests contain `{"uuids":[...]}`. Version 1 responses contain `{"version":1,"servers":[{"uuid":"...","instance":null}]}` for application servers or an instance object with `kind` (`container` or `virtual_machine`) and an OS image alias. Every requested UUID must have exactly one entry. Access is restricted to servers belonging to the authenticated node.

The migration adopts the previous native fork's `native_instance` metadata when present and rejects conflicting values. The legacy column remains intact. Back up the database before moving an existing native panel to stock panel plus this extension. The down migration refuses to drop metadata while native instances exist.

The extension uses form transforms for creation and editing, and supported frontend overrides for creation requests, edit metadata hydration and the native terminal. Application servers retain the original creation and console implementations. Native instance type and image changes require a new server.

Terminal code is adapted from the Calagopus panel under its MIT license. Build outputs and local validation harnesses are not part of the extension package.
