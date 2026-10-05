import type { UseFormReturnType } from '@mantine/form';
import { createContext, useContext, useEffect, useState } from 'react';
import { z } from 'zod';
import { httpErrorToHuman } from '@/api/axios.ts';
import Alert from '@/elements/feedback/Alert.tsx';
import Select from '@/elements/input/Select.tsx';
import Stack from '@/elements/layout/Stack.tsx';
import { useToast } from '@/providers/ToastProvider.tsx';
import { getNodeImages, getNodeRuntime, imageSchema, type NativeInstance, runtimeSchema } from './api.ts';
import { useConfigurationForm } from './ConfigurationScope.tsx';

export const EditingInstance = createContext<NativeInstance | null>(null);
export type InstanceForm = UseFormReturnType<Record<string, unknown>>;

export default function InstanceControls({ form, editing = false }: { form: InstanceForm; editing?: boolean }) {
  useConfigurationForm(form);
  const { addToast } = useToast();
  const existing = useContext(EditingInstance);
  const [node, setNode] = useState(String(form.getValues().nodeUuid ?? ''));
  const [instance, setInstance] = useState<NativeInstance | null>(() => (editing ? existing : null));
  const [runtime, setRuntime] = useState<z.infer<typeof runtimeSchema> | null>(null);
  const [images, setImages] = useState<z.infer<typeof imageSchema>[]>([]);
  const [loading, setLoading] = useState(false);
  form.watch('nodeUuid', ({ value }) => {
    if (!editing) setNode(String(value ?? ''));
  });

  useEffect(() => {
    if (editing) form.setFieldValue('incusInstance', existing);
  }, [editing, existing]);

  useEffect(() => {
    if (editing) return;
    let active = true;
    setRuntime(null);
    setImages([]);
    if (form.getValues().incusInstance) {
      form.setFieldValue('incusInstance', null);
      form.setFieldValue('eggUuid', '');
      form.setFieldValue('image', '');
      form.setFieldValue('startup', '');
      form.setFieldValue('skipInstaller', false);
      setInstance(null);
    }
    if (!node) return;
    setLoading(true);
    getNodeRuntime(node)
      .then(async (value) => {
        if (!active) return;
        setRuntime(value);
        if (value?.backend === 'incus') {
          const catalog = await getNodeImages(node);
          if (active) setImages(catalog);
        }
      })
      .catch((error) => {
        if (active) addToast(httpErrorToHuman(error), 'error');
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [node, editing]);

  if (editing)
    return existing ? (
      <Alert>
        Native {existing.kind === 'virtual_machine' ? 'virtual machine' : 'OS container'} · {existing.image}. Changing
        the instance type or image requires a new server.
      </Alert>
    ) : null;
  if (runtime?.backend !== 'incus') return null;
  if (!runtime.panelExtension)
    return <Alert>Enable runtime.incus.panel_extension in Wings and restart the daemon to create OS instances.</Alert>;

  const chooseKind = (kind: 'application' | NativeInstance['kind']) => {
    const next = kind === 'application' ? null : { kind, image: '', config: {} };
    setInstance(next);
    form.setFieldValue('incusInstance', next);
    form.setFieldValue('eggUuid', next ? '7f9047ea-14c8-4f8f-a1de-f267ea740111' : '');
    form.setFieldValue('startup', next ? '/sbin/init' : '');
    form.setFieldValue('image', '');
    form.setFieldValue('skipInstaller', !!next);
    form.setFieldValue('variables', []);
    if (next) {
      form.setFieldValue('hugepagesPassthroughEnabled', false);
      form.setFieldValue('kvmPassthroughEnabled', false);
    }
  };

  return (
    <Stack>
      <Select
        label='Instance type'
        allowDeselect={false}
        disabled={loading}
        value={instance?.kind ?? 'application'}
        data={[
          { value: 'application', label: 'Application container' },
          ...(runtime.systemContainers ? [{ value: 'container', label: 'OS container' }] : []),
          ...(runtime.virtualMachines ? [{ value: 'virtual_machine', label: 'Virtual machine' }] : []),
        ]}
        onChange={(value) => {
          if (value) chooseKind(value as 'application' | NativeInstance['kind']);
        }}
      />
      {instance && (
        <Select
          label='Operating system image'
          description={runtime.imageServer ?? undefined}
          searchable
          allowDeselect={false}
          disabled={loading}
          value={instance.image || null}
          error={form.errors.incusInstance}
          data={images
            .filter((image) => image.kind === instance.kind)
            .map((image) => ({
              value: image.alias,
              label: `${image.label} (${image.alias})`,
            }))}
          onChange={(image) => {
            const next = {
              ...instance,
              config: (form.getValues().incusInstance as NativeInstance).config,
              image: image ?? '',
            };
            setInstance(next);
            form.setFieldValue('incusInstance', next);
            form.setFieldValue('image', next.image);
          }}
        />
      )}
    </Stack>
  );
}
