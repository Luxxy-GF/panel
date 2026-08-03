import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/api-transform.ts';
import { fullUserSchema } from '@/lib/schemas/user.ts';

interface Response {
  user: z.infer<typeof fullUserSchema>;
}

export default async (token: string): Promise<Response> => {
  const { data } = await axiosInstance.post('/api/auth/verify-email/verify', { token });
  return { ...data, user: parseFromApi(fullUserSchema, data.user) };
};
