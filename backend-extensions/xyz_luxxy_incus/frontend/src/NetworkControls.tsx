import { useEffect, useState } from 'react';
import type { z } from 'zod';
import { httpErrorToHuman } from '@/api/axios.ts';
import Alert from '@/elements/feedback/Alert.tsx';
import Select from '@/elements/input/Select.tsx';
import TextInput from '@/elements/input/TextInput.tsx';
import Stack from '@/elements/layout/Stack.tsx';
import { useToast } from '@/providers/ToastProvider.tsx';
import { getIpPools, type networkRequestSchema, type poolSchema } from './api.ts';
import type { InstanceForm } from './InstanceControls.tsx';

export default function NetworkControls({
  form,
  node,
  enabled,
}: {
  form: InstanceForm;
  node: string;
  enabled: boolean;
}) {
  const { addToast } = useToast();
  const [pools, setPools] = useState<z.infer<typeof poolSchema>[]>([]);
  const [request, setRequest] = useState<z.infer<typeof networkRequestSchema> | null>(null);
  const [loading, setLoading] = useState(false);
  useEffect(() => {
    let active = true;
    setPools([]);
    setRequest(null);
    form.setFieldValue('incusNetworkRequest', null);
    if (!enabled || !node) return;
    setLoading(true);
    getIpPools(node)
      .then((pools) => {
        if (active) setPools(pools);
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
  }, [node, enabled]);
  const update = (next: typeof request) => {
    setRequest(next);
    form.setFieldValue('incusNetworkRequest', next);
    if (next) {
      form.setFieldValue('allocationUuid', null);
      form.setFieldValue('allocationUuids', []);
    }
  };
  const pool = pools.find((pool) => pool.uuid === request?.poolUuid);
  return (
    <Stack>
      <Select
        label='Networking'
        value={request?.poolUuid ?? 'bridge'}
        disabled={loading}
        allowDeselect={false}
        data={[
          { value: 'bridge', label: 'Managed bridge · port allocations' },
          ...pools.map((pool) => ({
            value: pool.uuid,
            label: `${pool.name} · ${pool.available} available`,
            disabled: pool.available === 0,
          })),
        ]}
        onChange={(value) =>
          update(!value || value === 'bridge' ? null : { poolUuid: value, address: null, mac: null })
        }
      />
      {!enabled && <Alert>Direct IP pools require updated Wings with Tundra disabled.</Alert>}
      {request && pool && (
        <>
          <Alert>
            {pool.config.start} – {pool.config.end} · {pool.config.subnet} · gateway {pool.config.gateway} ·{' '}
            {pool.config.parent}
            {pool.config.vlan && ` · VLAN ${pool.config.vlan}`}. No NAT or port forwards are created. The host cannot
            reach the guest directly; configure the firewall in the guest or upstream. Linux with systemd or OpenRC and
            iproute2 is required.
          </Alert>
          <TextInput
            label='IPv4 address'
            placeholder='Automatically select an available IP'
            value={request.address ?? ''}
            description='Choose an address from this pool, or leave empty for automatic assignment.'
            error={form.errors.incusNetworkRequest}
            onChange={(event) => update({ ...request, address: event.currentTarget.value || null })}
          />
          <TextInput
            label='MAC address'
            placeholder='Generate automatically'
            value={request.mac ?? ''}
            description='Optional MAC address assigned by your provider. The network must allow guest MAC addresses.'
            onChange={(event) => update({ ...request, mac: event.currentTarget.value || null })}
          />
        </>
      )}
    </Stack>
  );
}
