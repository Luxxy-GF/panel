import originalCreateServer from '@/api/admin/servers/createServer.ts';
import { axiosInstance } from '@/api/axios.ts';
import { adminServerCreateSchema, adminServerSchema } from '@/lib/schemas/admin/servers.ts';
import { formExtensionSchemas, parseFromApi, serializeForApi } from '@/lib/serialization/api-transform.ts';
import { metadataSchema, networkRequestSchema } from './api.ts';

const createServer: typeof originalCreateServer = async (values) => {
  const { incusInstance } = metadataSchema.parse(values);
  if (!incusInstance) return originalCreateServer(values);
  const payload = serializeForApi(
    adminServerCreateSchema,
    values,
    formExtensionSchemas('admin.servers.create'),
  ) as Record<string, unknown>;
  const request = (values as Record<string, unknown>).incusNetworkRequest;
  delete payload.incus_instance;
  delete payload.incus_network_request;
  if (request) {
    payload.network = serializeForApi(networkRequestSchema, networkRequestSchema.parse(request));
    payload.allocation_uuid = null;
    payload.allocation_uuids = [];
  }
  payload.instance = incusInstance;
  payload.egg_uuid = '7f9047ea-14c8-4f8f-a1de-f267ea740111';
  payload.variables = {};
  const { data } = await axiosInstance.post('/api/admin/incus/servers', payload);
  return parseFromApi(adminServerSchema, data.server);
};

export default createServer;
