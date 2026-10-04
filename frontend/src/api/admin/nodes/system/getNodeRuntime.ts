import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/serialization/api-transform.ts';

export const nodeRuntimeSchema = z.object({
  backend: z.string(),
  systemContainers: z.boolean(),
  virtualMachines: z.boolean(),
  imageServer: z.string().nullable(),
});

export type NodeRuntime = z.infer<typeof nodeRuntimeSchema>;

export default async (uuid: string): Promise<NodeRuntime | null> => {
  const { data } = await axiosInstance.get(`/api/admin/nodes/${uuid}/system/runtime`);
  return data === null ? null : parseFromApi(nodeRuntimeSchema, data);
};
