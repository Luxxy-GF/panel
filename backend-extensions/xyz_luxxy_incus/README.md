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

## Instance configuration

The extension replaces the create form's Variables card with Incus configuration for native OS instances and adds the same section to the edit form. Switches, integer inputs, selectors, and multiline fields cover the Incus 7.0.1 LTS native instance option catalog. Container and VM options are filtered by applicability. Dynamic families such as `environment.*`, `linux.sysctl.*`, `systemd.credential.*`, and `user.*` accept named keys.

Stop an existing instance before editing configuration. The update endpoint requires `servers.update`, validates the values, and saves metadata in the standard server transaction before syncing Wings. Clearing a value removes that override; Wings tracks extension-managed keys to remove previous values on the next sync/start. Unspecified values retain Incus defaults, except Wings' isolated container ID mapping default.

The normal resource controls own `limits.cpu`, `limits.cpu.allowance`, `limits.memory`, `limits.memory.swap`, and `limits.disk.priority`. Wings owns `boot.autostart`. These appear read-only here. Incus-generated `volatile.*`, application-only `oci.*`, and internal `user.wings.*` keys cannot be edited. Device, project, network, storage-pool, and daemon options are separate from instance config and are not part of this section. Hardware-dependent settings remain subject to Incus validation; configuring migration or snapshots does not add missing Wings transfer or backup support.

Full configuration is available only through admin and authenticated node metadata. Client responses include instance type and image without configuration values. Update both the panel extension and Wings before using these controls.

The catalog descriptions are derived from Incus v7.0.1 `internal/server/metadata/configuration.json`, under the Apache 2.0 license reproduced in `LICENSE.incus`. The metadata was narrowed to native instance options and annotated with Wings-managed settings. Reference: https://linuxcontainers.org/incus/docs/stable-7.0/reference/instance_options/.
