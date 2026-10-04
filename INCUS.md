# Incus native instances

Use this panel branch together with the Wings `feat/incus-native-instances` branch. Apply the panel's database migrations before creating servers; the migration adds a native-instance field and a built-in OS template.

On server creation, select an Incus node. The panel queries that node's runtime capabilities and configured Simple Streams catalog. The instance selector offers application containers, system containers, and virtual machines when KVM is available. Select an OS image for either native type. Docker nodes and older Wings nodes retain the application-container creation flow.

Native instances use persistent Incus root disks, the existing allocation and resource inputs, an interactive root console, and guest filesystem access through Incus SFTP. Wings hosts need `sshfs` and `fuse3`. VM file and shell access require a running guest with the Incus agent. VM disk files with the default `dir` pool are stored beneath the Wings root directory; OS containers use root filesystem directories.

Native instance type and image cannot be changed through ordinary server updates. Egg installer scripts, custom startup commands, and environment variables are not used to boot an OS. Tundra private networking is not supported for native OS instances; VM nodes must disable Tundra. Full native backup/export, transfer, extra mounts, and OS reinstallation are not yet supported by the companion Wings branch. Its APIs reject game-data archive operations for OS instances. Use the existing application-container mode for those workflows.
