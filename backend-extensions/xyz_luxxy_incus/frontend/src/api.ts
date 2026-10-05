import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/serialization/api-transform.ts';

export const instanceSchema = z.object({
  kind: z.enum(['container', 'virtual_machine']),
  image: z.string().min(1).max(255),
});
export type NativeInstance = z.infer<typeof instanceSchema>;
export const metadataSchema = z.object({ incusInstance: instanceSchema.nullable().default(null) });
export const runtimeSchema = z.object({
  backend: z.string(),
  systemContainers: z.boolean(),
  virtualMachines: z.boolean(),
  imageServer: z.string().nullable(),
  panelExtension: z.boolean().default(false),
});
export const imageSchema = z.object({ alias: z.string(), label: z.string(), kind: instanceSchema.shape.kind });

export async function getNodeRuntime(uuid: string) {
  const { data } = await axiosInstance.get(`/api/admin/incus/nodes/${uuid}/runtime`);
  return data === null ? null : parseFromApi(runtimeSchema, data);
}

export async function getNodeImages(uuid: string) {
  const { data } = await axiosInstance.get(`/api/admin/incus/nodes/${uuid}/images`);
  return z.array(imageSchema).parse(data);
}
