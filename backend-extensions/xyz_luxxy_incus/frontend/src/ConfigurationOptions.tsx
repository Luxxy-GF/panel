import { Accordion, Button, Group, Text } from '@mantine/core';
import { useState } from 'react';
import NumberInput from '@/elements/input/NumberInput.tsx';
import Select from '@/elements/input/Select.tsx';
import Switch from '@/elements/input/Switch.tsx';
import TextArea from '@/elements/input/TextArea.tsx';
import TextInput from '@/elements/input/TextInput.tsx';
import Stack from '@/elements/layout/Stack.tsx';
import type { NativeInstance } from './api.ts';
import catalog from './config-options.json';
import type { InstanceForm } from './InstanceControls.tsx';

const titles: Record<string, string> = {
  security: 'Security and nesting',
  boot: 'Boot and shutdown',
  'cloud-init': 'Cloud-init',
  'resource-limits': 'Additional resource limits',
  miscellaneous: 'System and environment',
  migration: 'Migration',
  nvidia: 'NVIDIA runtime',
  raw: 'Raw configuration',
  snapshots: 'Snapshots',
};
const labels: Record<string, string> = {
  'security.nesting': 'Enable nesting',
  'security.privileged': 'Privileged container',
  'security.secureboot': 'Secure boot',
  'security.csm': 'Legacy BIOS boot',
  'security.guestapi': 'Guest API',
};

export default function ConfigurationOptions({ form, instance }: { form: InstanceForm; instance: NativeInstance }) {
  const [search, setSearch] = useState('');
  const [pattern, setPattern] = useState('environment.*');
  const [key, setKey] = useState('');
  const [value, setValue] = useState('');
  const [error, setError] = useState<string | null>(null);
  const config = instance.config ?? {};
  const setOption = (name: string, next?: string) => {
    const current = form.getValues().incusInstance as NativeInstance;
    const updated = { ...current.config };
    if (next === undefined || next === '') delete updated[name];
    else updated[name] = next;
    form.setFieldValue('incusInstance', { ...current, config: updated });
  };
  const options = catalog.filter((option) => option.kinds.includes(instance.kind));
  const wildcardOptions = options.filter((option) => option.key.endsWith('*'));
  const groups = [...new Set(options.map((option) => option.group))];
  const matchesSearch = (option: (typeof catalog)[number]) =>
    `${option.key} ${option.description}`.toLowerCase().includes(search.toLowerCase());
  const field = (option: (typeof catalog)[number], name = option.key) => {
    const description = `${name} · ${option.description}${option.default ? ` · Default: ${option.default}` : ''}${option.condition ? ` · ${option.condition}` : ''}`;
    const props = { label: labels[name] ?? name, description };
    const current = config[name];
    let input;
    if (option.managed)
      input = <TextInput {...props} value='Managed by Wings resource limits or startup settings' disabled />;
    else if (option.type === 'bool')
      input = (
        <Switch
          {...props}
          checked={current === undefined ? option.default === 'true' : current === 'true'}
          onChange={(event) => setOption(name, String(event.currentTarget.checked))}
        />
      );
    else if (option.choices.length)
      input = (
        <Select
          {...props}
          clearable
          data={option.choices}
          value={current ?? null}
          placeholder={option.default || 'Inherit default'}
          onChange={(next) => setOption(name, next ?? undefined)}
        />
      );
    else if (option.type === 'integer' || option.type === 'int64')
      input = (
        <NumberInput
          {...props}
          allowDecimal={false}
          value={current ?? ''}
          placeholder={option.default || 'Inherit default'}
          onChange={(next) => setOption(name, String(next))}
        />
      );
    else if (
      option.type === 'blob' ||
      option.group === 'cloud-init' ||
      name.startsWith('security.syscalls.') ||
      option.key.endsWith('*')
    )
      input = (
        <TextArea
          {...props}
          autosize
          minRows={3}
          maxRows={12}
          value={current ?? ''}
          placeholder={option.default || 'Inherit default'}
          onChange={(event) => setOption(name, event.currentTarget.value)}
        />
      );
    else
      input = (
        <TextInput
          {...props}
          value={current ?? ''}
          placeholder={option.default || 'Inherit default'}
          onChange={(event) => setOption(name, event.currentTarget.value)}
        />
      );
    return (
      <Stack key={name} gap='xs'>
        {input}
        {current !== undefined && !option.managed && (
          <Button variant='subtle' size='compact-xs' onClick={() => setOption(name)}>
            Reset to default
          </Button>
        )}
      </Stack>
    );
  };
  const customEntries = Object.entries(config).filter(
    ([name]) =>
      wildcardOptions.some((option) => name.startsWith(option.key.slice(0, -1))) &&
      !options.some((option) => option.key === name),
  );
  return (
    <Stack>
      <Text size='sm' c='dimmed'>
        Incus 7.0 LTS instance options. Only changed values are stored. Stop the instance before editing these settings;
        changes apply on its next start. Hardware, image, and storage support still determine which options Incus
        accepts.
      </Text>
      <TextInput label='Search options' value={search} onChange={(event) => setSearch(event.currentTarget.value)} />
      <Accordion multiple defaultValue={['security']}>
        {groups.map((group) => {
          const entries = options.filter(
            (option) => option.group === group && !option.key.endsWith('*') && matchesSearch(option),
          );
          if (!entries.length) return null;
          return (
            <Accordion.Item key={group} value={group}>
              <Accordion.Control>{titles[group] ?? group}</Accordion.Control>
              <Accordion.Panel>
                <div className='grid grid-cols-1 xl:grid-cols-2 gap-4'>{entries.map((option) => field(option))}</div>
              </Accordion.Panel>
            </Accordion.Item>
          );
        })}
      </Accordion>
      <Text fw={600}>Environment, sysctl, credentials, and custom keys</Text>
      {customEntries.map(([name]) => {
        const option = wildcardOptions.find((option) => name.startsWith(option.key.slice(0, -1)));
        return option ? field(option, name) : null;
      })}
      <Group align='end' grow>
        <Select
          label='Key family'
          allowDeselect={false}
          data={wildcardOptions.map((option) => option.key)}
          value={pattern}
          onChange={(next) => {
            if (next) {
              setPattern(next);
              setKey('');
            }
          }}
        />
        <TextInput
          label='Full configuration key'
          placeholder={pattern.replace('*', 'NAME')}
          value={key}
          error={error}
          onChange={(event) => {
            setKey(event.currentTarget.value);
            setError(null);
          }}
        />
      </Group>
      <TextArea
        label='Value'
        value={value}
        autosize
        minRows={2}
        onChange={(event) => setValue(event.currentTarget.value)}
      />
      <Button
        variant='light'
        onClick={() => {
          const prefix = pattern.slice(0, -1);
          if (
            !key.startsWith(prefix) ||
            key.length <= prefix.length ||
            !/^[a-zA-Z0-9._-]{1,255}$/.test(key) ||
            key.startsWith('user.wings.') ||
            !value
          ) {
            setError('Enter a full key in the selected family and a non-empty value. user.wings.* is reserved.');
            return;
          }
          setOption(key, value);
          setKey('');
          setValue('');
        }}
      >
        Add configuration key
      </Button>
    </Stack>
  );
}
