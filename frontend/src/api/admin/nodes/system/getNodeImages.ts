import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';

const imageSchema = z.object({
  alias: z.string(),
  label: z.string(),
  kind: z.enum(['container', 'virtual_machine']),
});

export type NativeImage = z.infer<typeof imageSchema>;

export default async (uuid: string): Promise<NativeImage[]> => {
  const { data } = await axiosInstance.get(`/api/admin/nodes/${uuid}/system/images`);
  return z.array(imageSchema).parse(data);
};
