// Zod schema fixtures
import { z } from 'zod';

const UserSchema = z.object({
  id: z.string().uuid(),
  email: z.string().email(),
  name: z.string().min(1).max(100),
  age: z.number().optional(),
});

const PostSchema = z.object({
  title: z.string(),
  content: z.string(),
  published: z.boolean().default(false),
});

export const ConfigSchema = z.object({
  port: z.number(),
  host: z.string(),
});
