import type { z } from 'zod';
import originalUpdateServer from '@/api/admin/servers/updateServer.ts';
import { axiosInstance } from '@/api/axios.ts';
import { adminServerUpdateSchema } from '@/lib/schemas/admin/servers.ts';
import { formExtensionSchemas, serializeForApi } from '@/lib/serialization/api-transform.ts';
import { metadataSchema } from './api.ts';

const updateServer: typeof originalUpdateServer = async (uuid, values) => {
  const { incusInstance } = metadataSchema.parse(values);
  if (!incusInstance) return originalUpdateServer(uuid, values);
  const payload = serializeForApi(
    adminServerUpdateSchema,
    values as z.infer<typeof adminServerUpdateSchema>,
    formExtensionSchemas('admin.servers.update'),
  ) as Record<string, unknown>;
  delete payload.incus_instance;
  payload.instance = incusInstance;
  await axiosInstance.patch(`/api/admin/incus/servers/${uuid}`, payload);
};

export default updateServer;
