import { Group, Stack } from '@mantine/core';
import { UseFormReturnType } from '@mantine/form';
import { useEffect } from 'react';
import { z } from 'zod';
import NumberInput from '@/elements/input/NumberInput.tsx';
import PasswordInput from '@/elements/input/PasswordInput.tsx';
import Switch from '@/elements/input/Switch.tsx';
import TextInput from '@/elements/input/TextInput.tsx';
import { adminSettingsEmailSmtpSchema } from '@/lib/schemas/admin/settings.ts';

export default function EmailSmtp({ form }: { form: UseFormReturnType<z.infer<typeof adminSettingsEmailSmtpSchema>> }) {
  useEffect(() => {
    form.setValues({
      host: form.values.host ?? '',
      port: form.values.port ?? 587,
      username: form.values.username ?? null,
      password: form.values.password ?? null,
      useTls: form.values.useTls ?? true,
      fromAddress: form.values.fromAddress ?? '',
      fromName: form.values.fromName ?? null,
    });
  }, []);

  return (
    <Stack mt='md'>
      <Group grow>
        <TextInput
          withAsterisk
          label='Host'
          placeholder='Host'
          key={form.key('host')}
          {...form.getInputProps('host')}
        />
        <NumberInput
          withAsterisk
          label='Port'
          placeholder='Port'
          min={0}
          key={form.key('port')}
          {...form.getInputProps('port')}
        />
      </Group>

      <Group grow>
        <Switch label='Use TLS' key={form.key('useTls')} {...form.getInputProps('useTls', { type: 'checkbox' })} />
        <Switch
          label='Skip Certificate Validation'
          key={form.key('skipCertValidation')}
          {...form.getInputProps('skipCertValidation', { type: 'checkbox' })}
        />
      </Group>

      <Group grow>
        <TextInput
          label='Username'
          placeholder='Username'
          key={form.key('username')}
          {...form.getInputProps('username')}
        />
        <PasswordInput
          label='Password'
          placeholder='Password'
          key={form.key('password')}
          {...form.getInputProps('password')}
        />
      </Group>

      <Group grow>
        <TextInput
          withAsterisk
          label='From Address'
          placeholder='From Address'
          key={form.key('fromAddress')}
          {...form.getInputProps('fromAddress')}
        />
        <TextInput
          label='From Name'
          placeholder='From Name'
          key={form.key('fromName')}
          {...form.getInputProps('fromName')}
        />
      </Group>
    </Stack>
  );
}
