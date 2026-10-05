ALTER TABLE "servers" ADD COLUMN IF NOT EXISTS "xyz_luxxy_incus_instance" jsonb;
INSERT INTO "nests" ("uuid", "name", "description", "author") VALUES
('7f9047ea-14c8-4f8f-a1de-f267ea740110', 'Operating systems', 'Native Incus OS instances', 'Calagopus') ON CONFLICT ("uuid") DO NOTHING;
INSERT INTO "nest_eggs" ("uuid", "nest_uuid", "author", "name", "description", "config_files", "config_startup", "config_stop", "config_script", "startup_commands", "force_outgoing_ip", "separate_port", "features", "docker_images", "file_denylist") VALUES
('7f9047ea-14c8-4f8f-a1de-f267ea740111', '7f9047ea-14c8-4f8f-a1de-f267ea740110', 'Calagopus', 'Native OS instance', 'Boots an Incus system container or virtual machine', '[]', '{"done":[],"strip_ansi":false}', '{"type":"signal","value":"SIGTERM"}', '{"container":"alpine:latest","entrypoint":"/bin/sh","content":""}', '{"Default":"/sbin/init"}', false, false, '{}', '{}', ARRAY['proc/**','sys/**','dev/**','run/**']) ON CONFLICT ("uuid") DO NOTHING;

DO $$ DECLARE conflict boolean; BEGIN
IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = current_schema() AND table_name = 'servers' AND column_name = 'native_instance') THEN
IF EXISTS (SELECT 1 FROM servers WHERE xyz_luxxy_incus_instance IS NOT NULL) THEN
EXECUTE 'SELECT EXISTS (SELECT 1 FROM servers WHERE native_instance IS NOT NULL AND xyz_luxxy_incus_instance IS NOT NULL AND native_instance::jsonb <> xyz_luxxy_incus_instance)' INTO conflict;
IF conflict THEN RAISE EXCEPTION 'conflicting legacy and extension Incus metadata'; END IF;
END IF;
EXECUTE 'UPDATE servers SET xyz_luxxy_incus_instance = native_instance::jsonb WHERE native_instance IS NOT NULL AND xyz_luxxy_incus_instance IS NULL';
END IF;
END $$;
