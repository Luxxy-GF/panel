ALTER TABLE "servers" ADD COLUMN "native_instance" json;
INSERT INTO "nests" ("uuid", "name", "description", "author") VALUES
('7f9047ea-14c8-4f8f-a1de-f267ea740110', 'Operating systems', 'Native Incus OS instances', 'Calagopus') ON CONFLICT ("uuid") DO NOTHING;
INSERT INTO "nest_eggs" ("uuid", "nest_uuid", "author", "name", "description", "config_files", "config_startup", "config_stop", "config_script", "startup_commands", "force_outgoing_ip", "separate_port", "features", "docker_images", "file_denylist") VALUES
('7f9047ea-14c8-4f8f-a1de-f267ea740111', '7f9047ea-14c8-4f8f-a1de-f267ea740110', 'Calagopus', 'Native OS instance', 'Boots an Incus system container or virtual machine', '[]', '{"done":[],"strip_ansi":false}', '{"type":"signal","value":"SIGTERM"}', '{"container":"alpine:latest","entrypoint":"/bin/sh","content":""}', '{"Default":"/sbin/init"}', false, false, '{}', '{}', ARRAY['proc/**','sys/**','dev/**','run/**']) ON CONFLICT ("uuid") DO NOTHING;
