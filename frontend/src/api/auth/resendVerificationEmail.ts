import { axiosInstance } from '@/api/axios.ts';

interface Payload {
  email: string;
  captcha?: string | null;
}

export default async (data: Payload): Promise<void> => {
  await axiosInstance.post('/api/auth/verify-email/resend', data);
};
