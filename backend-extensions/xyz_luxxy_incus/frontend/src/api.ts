import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/serialization/api-transform.ts';

export const externalNetworkSchema = z.object({
  pool_uuid: z.uuid(),
  parent: z.string(),
  vlan: z.number().nullable(),
  mtu: z.number().nullable(),
  mode: z.string(),
  gvrp: z.boolean(),
  address: z.ipv4(),
  prefix: z.number(),
  gateway: z.ipv4(),
  gateway_onlink: z.boolean(),
  dns: z.array(z.ipv4()),
  mac: z.string().nullable(),
});
export const networkRequestSchema = z.object({
  poolUuid: z.uuid(),
  address: z.ipv4().nullable(),
  mac: z.string().nullable(),
});

export const instanceSchema = z.object({
  network: externalNetworkSchema.nullable().default(null),
  kind: z.enum(['container', 'virtual_machine']),
  image: z.string().min(1).max(255),
  config: z.record(z.string().max(255), z.string().min(1).max(65536)).default({}),
});
export type NativeInstance = z.infer<typeof instanceSchema>;
export const metadataSchema = z.object({
  incusAddress: z.ipv4().nullable().default(null),
  incusInstance: instanceSchema.nullable().default(null),
});
export const runtimeSchema = z.object({
  backend: z.string(),
  directNetworking: z.boolean().default(false),
  systemContainers: z.boolean(),
  virtualMachines: z.boolean(),
  imageServer: z.string().nullable(),
  panelExtension: z.boolean().default(false),
});
export const imageSchema = z.object({
  alias: z.string(),
  label: z.string(),
  kind: instanceSchema.shape.kind,
});

export async function getNodeRuntime(uuid: string) {
  const { data } = await axiosInstance.get(`/api/admin/incus/nodes/${uuid}/runtime`);
  return data === null ? null : parseFromApi(runtimeSchema, data);
}

export async function getNodeImages(uuid: string) {
  const { data } = await axiosInstance.get(`/api/admin/incus/nodes/${uuid}/images`);
  return z.array(imageSchema).parse(data);
}

export const poolSettingsSchema = z.object({
  parent: z.string().min(1),
  vlan: z.number().int().min(1).max(4094).nullable(),
  mtu: z.number().int().min(576).max(65535).nullable(),
  mode: z.enum(['bridge', 'private', 'vepa']),
  gvrp: z.boolean(),
  subnet: z.string().min(1),
  start: z.ipv4(),
  end: z.ipv4(),
  gateway: z.ipv4(),
  gatewayOnlink: z.boolean(),
  dns: z.array(z.ipv4()).min(1).max(4),
});
export const poolSchema = z.object({
  uuid: z.uuid(),
  name: z.string(),
  config: poolSettingsSchema,
  total: z.number(),
  available: z.number(),
});
export const inventorySchema = z.object({
  interfaces: z.array(z.object({ name: z.string(), mtu: z.number() })),
  leases: z.array(z.object({ serverUuid: z.uuid(), address: z.ipv4() })),
});
export const poolAddressSchema = z.object({
  address: z.ipv4(),
  serverUuid: z.uuid().nullable(),
  serverName: z.string().nullable(),
});

export async function getIpPools(node: string) {
  const { data } = await axiosInstance.get(`/api/admin/incus/nodes/${node}/ip-pools`);
  return parseFromApi(z.array(poolSchema), data);
}
export async function getNetworkInventory(node: string) {
  const { data } = await axiosInstance.get(`/api/admin/incus/nodes/${node}/network`);
  return parseFromApi(inventorySchema, data);
}
export async function getPoolAddresses(node: string, pool: string, offset: number) {
  const { data } = await axiosInstance.get(`/api/admin/incus/nodes/${node}/ip-pools/${pool}/addresses`, {
    params: { offset },
  });
  return parseFromApi(z.array(poolAddressSchema), data);
}
