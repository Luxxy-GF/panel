import { z } from 'zod';

export const announcementType = z.enum(['info', 'warning', 'error']);

export const announcementSchema = z.object({
  uuid: z.string(),
  type: announcementType,
  title: z.string(),
  titleTranslations: z.record(z.string(), z.string()),
  content: z.string(),
  contentTranslations: z.record(z.string(), z.string()),
});
