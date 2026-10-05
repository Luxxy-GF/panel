import { useCallback, useEffect, useState } from 'react';
import { z } from 'zod';
import { axiosInstance, httpErrorToHuman } from '@/api/axios.ts';
import Button from '@/elements/buttons/Button.tsx';
import AdminSubContentContainer from '@/elements/containers/AdminSubContentContainer.tsx';
import TitleCard from '@/elements/data-display/TitleCard.tsx';
import Alert from '@/elements/feedback/Alert.tsx';
import NumberInput from '@/elements/input/NumberInput.tsx';
import Select from '@/elements/input/Select.tsx';
import Switch from '@/elements/input/Switch.tsx';
import TextInput from '@/elements/input/TextInput.tsx';
import Group from '@/elements/layout/Group.tsx';
import Stack from '@/elements/layout/Stack.tsx';
import { adminNodeSchema } from '@/lib/schemas/admin/nodes.ts';
import { serializeForApi } from '@/lib/serialization/api-transform.ts';
import { useToast } from '@/providers/ToastProvider.tsx';
import {
  getIpPools,
  getNetworkInventory,
  getPoolAddresses,
  inventorySchema,
  poolAddressSchema,
  poolSchema,
  poolSettingsSchema,
} from './api.ts';

const empty = {
  parent: '',
  vlan: null,
  mtu: null,
  mode: 'bridge' as const,
  gvrp: false,
  subnet: '',
  start: '',
  end: '',
  gateway: '',
  gatewayOnlink: false,
  dns: [] as string[],
};

export default function IpPools({ node }: { node: z.infer<typeof adminNodeSchema> }) {
  const { addToast } = useToast();
  const [pools, setPools] = useState<z.infer<typeof poolSchema>[]>([]);
  const [inventory, setInventory] = useState<z.infer<typeof inventorySchema> | null>(null);
  const [config, setConfig] = useState<z.infer<typeof poolSettingsSchema>>(empty);
  const [name, setName] = useState('');
  const [dns, setDns] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [view, setView] = useState<{ pool: string; offset: number } | null>(null);
  const [addresses, setAddresses] = useState<z.infer<typeof poolAddressSchema>[]>([]);
  const refresh = useCallback(async () => {
    const [pools, inventory] = await Promise.all([getIpPools(node.uuid), getNetworkInventory(node.uuid)]);
    setPools(pools);
    setInventory(inventory);
    setError(null);
  }, [node.uuid]);
  useEffect(() => {
    refresh().catch((error) => setError(httpErrorToHuman(error)));
  }, [refresh]);
  useEffect(() => {
    let active = true;
    setAddresses([]);
    if (view)
      getPoolAddresses(node.uuid, view.pool, view.offset)
        .then((addresses) => {
          if (active) setAddresses(addresses);
        })
        .catch((error) => addToast(httpErrorToHuman(error), 'error'));
    return () => {
      active = false;
    };
  }, [node.uuid, view]);
  const run = async (action: () => Promise<void>) => {
    setBusy(true);
    try {
      await action();
      await refresh();
    } catch (error) {
      addToast(httpErrorToHuman(error), 'error');
    } finally {
      setBusy(false);
    }
  };
  return (
    <AdminSubContentContainer title='Incus IP Pools'>
      <Stack>
        {error && <Alert>{error}</Alert>}
        <Alert>
          Assign external IPv4 addresses to Linux OS containers or VMs on a parent NIC, with optional VLAN tagging and
          no managed bridge. Your network must permit guest MAC addresses. Deleted servers keep their reservations until
          reconciliation confirms Incus deletion.
        </Alert>
        <Group>
          <Button loading={busy} onClick={() => run(refresh)}>
            Refresh
          </Button>
          <Button
            variant='outline'
            loading={busy}
            disabled={!inventory}
            onClick={() =>
              run(async () => {
                const { data } = await axiosInstance.post(`/api/admin/incus/nodes/${node.uuid}/ip-pools/reconcile`);
                addToast(`Released ${data.released} deleted-instance reservations.`, 'success');
                setView(null);
              })
            }
          >
            Reconcile deleted instances
          </Button>
        </Group>
        {pools.map((pool) => (
          <TitleCard key={pool.uuid} title={pool.name}>
            <Stack>
              <div>
                {pool.config.start} – {pool.config.end} · {pool.config.subnet} · {pool.config.parent}
                {pool.config.vlan && ` · VLAN ${pool.config.vlan}`}
              </div>
              <div>
                Gateway {pool.config.gateway} · DNS {pool.config.dns.join(', ')} · {pool.available} / {pool.total}{' '}
                available
              </div>
              <Group>
                <Button variant='outline' onClick={() => setView({ pool: pool.uuid, offset: 0 })}>
                  View addresses
                </Button>
                <Button
                  color='red'
                  variant='outline'
                  disabled={pool.available !== pool.total}
                  loading={busy}
                  onClick={() =>
                    run(async () => {
                      await axiosInstance.delete(`/api/admin/incus/nodes/${node.uuid}/ip-pools/${pool.uuid}`);
                      setView(null);
                    })
                  }
                >
                  Delete empty pool
                </Button>
              </Group>
              {view?.pool === pool.uuid && (
                <>
                  <table className='w-full text-left text-sm'>
                    <thead>
                      <tr>
                        <th>IPv4</th>
                        <th>Assignment</th>
                      </tr>
                    </thead>
                    <tbody>
                      {addresses.map((address) => (
                        <tr key={address.address}>
                          <td className='py-1'>{address.address}</td>
                          <td>
                            {address.serverUuid ? (
                              address.serverName ? (
                                <a href={`/admin/servers/${address.serverUuid}`}>{address.serverName}</a>
                              ) : (
                                `Reserved until reconciled · ${address.serverUuid}`
                              )
                            ) : (
                              'Available'
                            )}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                  <Group>
                    <Button
                      disabled={view.offset === 0}
                      onClick={() => setView({ ...view, offset: Math.max(0, view.offset - 100) })}
                    >
                      Previous
                    </Button>
                    <span>
                      {view.offset + 1} – {view.offset + addresses.length} of {pool.total}
                    </span>
                    <Button
                      disabled={view.offset + addresses.length >= pool.total}
                      onClick={() => setView({ ...view, offset: view.offset + 100 })}
                    >
                      Next
                    </Button>
                  </Group>
                </>
              )}
            </Stack>
          </TitleCard>
        ))}
        <TitleCard title='Create IP Pool'>
          <Stack>
            <TextInput label='Name' value={name} onChange={(event) => setName(event.currentTarget.value)} />
            <Select
              label='Parent interface'
              searchable
              allowDeselect={false}
              placeholder='Physical NIC or bond'
              value={config.parent || null}
              data={
                inventory?.interfaces.map((item) => ({ value: item.name, label: `${item.name} · MTU ${item.mtu}` })) ??
                []
              }
              onChange={(value) => setConfig({ ...config, parent: value ?? '' })}
            />
            <Group grow>
              <NumberInput
                label='VLAN ID'
                description='Empty for untagged traffic'
                min={1}
                max={4094}
                value={config.vlan ?? ''}
                onChange={(value) => setConfig({ ...config, vlan: value === '' ? null : Number(value) })}
              />
              <NumberInput
                label='MTU'
                description='Empty to inherit'
                min={576}
                max={65535}
                value={config.mtu ?? ''}
                onChange={(value) => setConfig({ ...config, mtu: value === '' ? null : Number(value) })}
              />
            </Group>
            <Select
              label='Macvlan mode'
              description='Bridge allows peer guests; private isolates them; VEPA needs switch reflective relay. These modes do not create a managed bridge.'
              value={config.mode}
              data={['bridge', 'private', 'vepa']}
              allowDeselect={false}
              onChange={(value) => setConfig({ ...config, mode: value as typeof config.mode })}
            />
            <Switch
              label='GVRP VLAN registration'
              checked={config.gvrp}
              onChange={(event) => setConfig({ ...config, gvrp: event.currentTarget.checked })}
            />
            <TextInput
              label='IPv4 subnet'
              placeholder='192.0.2.0/24'
              value={config.subnet}
              onChange={(event) => setConfig({ ...config, subnet: event.currentTarget.value })}
            />
            <Group grow>
              <TextInput
                label='First assignable IPv4'
                placeholder='192.0.2.10'
                value={config.start}
                onChange={(event) => setConfig({ ...config, start: event.currentTarget.value })}
              />
              <TextInput
                label='Last assignable IPv4'
                placeholder='192.0.2.50'
                value={config.end}
                onChange={(event) => setConfig({ ...config, end: event.currentTarget.value })}
              />
            </Group>
            <TextInput
              label='Gateway'
              placeholder='192.0.2.1'
              value={config.gateway}
              onChange={(event) => setConfig({ ...config, gateway: event.currentTarget.value })}
            />
            <Switch
              label='Gateway is on-link'
              description='Enable for a gateway outside the subnet, including provider-routed /32 addresses.'
              checked={config.gatewayOnlink}
              onChange={(event) => setConfig({ ...config, gatewayOnlink: event.currentTarget.checked })}
            />
            <TextInput
              label='DNS servers'
              description='One through four IPv4 addresses, separated by commas or spaces'
              value={dns}
              onChange={(event) => setDns(event.currentTarget.value)}
            />
            <Button
              loading={busy}
              disabled={!inventory || !name.trim()}
              onClick={() =>
                run(async () => {
                  const parsed = poolSettingsSchema.parse({ ...config, dns: dns.split(/[\s,]+/).filter(Boolean) });
                  await axiosInstance.post(`/api/admin/incus/nodes/${node.uuid}/ip-pools`, {
                    name: name.trim(),
                    config: serializeForApi(poolSettingsSchema, parsed),
                  });
                  setName('');
                  setConfig(empty);
                  setDns('');
                })
              }
            >
              Create pool
            </Button>
          </Stack>
        </TitleCard>
      </Stack>
    </AdminSubContentContainer>
  );
}
