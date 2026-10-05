DO $$ BEGIN
IF EXISTS (SELECT 1 FROM xyz_luxxy_incus_ip_addresses WHERE server_uuid IS NOT NULL) THEN
RAISE EXCEPTION 'IP leases remain reserved; delete their instances and reconcile pools before uninstalling';
END IF;
END $$;
DROP TABLE xyz_luxxy_incus_ip_addresses;
DROP TABLE xyz_luxxy_incus_ip_pools;
