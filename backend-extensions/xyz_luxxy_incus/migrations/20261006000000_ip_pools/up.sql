CREATE TABLE xyz_luxxy_incus_ip_pools (
    uuid uuid PRIMARY KEY,
    node_uuid uuid NOT NULL REFERENCES nodes(uuid) ON DELETE RESTRICT,
    name text NOT NULL,
    config jsonb NOT NULL,
    UNIQUE (uuid, node_uuid)
);
CREATE TABLE xyz_luxxy_incus_ip_addresses (
    pool_uuid uuid NOT NULL,
    node_uuid uuid NOT NULL,
    address inet NOT NULL CHECK (family(address) = 4 AND masklen(address) = 32),
    server_uuid uuid,
    mac text,
    PRIMARY KEY (node_uuid, address),
    FOREIGN KEY (pool_uuid, node_uuid) REFERENCES xyz_luxxy_incus_ip_pools(uuid, node_uuid) ON DELETE CASCADE
);
CREATE UNIQUE INDEX xyz_luxxy_incus_ip_address_server ON xyz_luxxy_incus_ip_addresses(server_uuid) WHERE server_uuid IS NOT NULL;

CREATE UNIQUE INDEX xyz_luxxy_incus_ip_address_mac ON xyz_luxxy_incus_ip_addresses(node_uuid, mac) WHERE mac IS NOT NULL;
