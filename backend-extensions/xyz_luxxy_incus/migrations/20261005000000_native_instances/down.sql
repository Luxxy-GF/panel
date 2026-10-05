DO $$ BEGIN
IF EXISTS (SELECT 1 FROM servers WHERE xyz_luxxy_incus_instance IS NOT NULL) THEN
RAISE EXCEPTION 'Native instances still depend on xyz.luxxy.incus; migrate or delete them before uninstalling';
END IF;
END $$;
ALTER TABLE servers DROP COLUMN xyz_luxxy_incus_instance;
