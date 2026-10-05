import { useContext, useState } from 'react';
import { makeComponentHookable } from 'shared';
import type { TitleCardProps } from '@/elements/data-display/TitleCard.tsx';
import OriginalTitleCard from '@/elements/data-display/TitleCard.tsx';
import { useTranslations } from '@/providers/TranslationProvider.tsx';
import ConfigurationOptions from './ConfigurationOptions.tsx';
import { ConfigurationContext, useInstance } from './ConfigurationScope.tsx';
import type { InstanceForm } from './InstanceControls.tsx';

const ConfigurationCard = (props: TitleCardProps) => {
  const scope = useContext(ConfigurationContext);
  if (!scope?.form) return <OriginalTitleCard {...props} />;
  return <NativeConfigurationCard {...props} form={scope.form} mode={scope.mode} />;
};

function NativeConfigurationCard({
  form,
  mode,
  ...props
}: TitleCardProps & { form: InstanceForm; mode: 'create' | 'update' }) {
  const instance = useInstance(form);
  const [poolSelected, setPoolSelected] = useState(!!form.getValues().incusNetworkRequest);
  form.watch('incusNetworkRequest', ({ value }) => setPoolSelected(!!value));
  const { t } = useTranslations();
  if (!instance) return <OriginalTitleCard {...props} />;
  if (
    (poolSelected || instance.network) &&
    props.title === t('pages.admin.servers.tabs.general.page.card.allocations', {})
  )
    return (
      <OriginalTitleCard {...props} title='Networking'>
        This instance uses its own IPv4 address from an IP pool. Port allocations and NAT are not required.
      </OriginalTitleCard>
    );
  const options = (
    <OriginalTitleCard title='Incus configuration' className='col-span-full'>
      <ConfigurationOptions form={form} instance={instance} />
    </OriginalTitleCard>
  );
  if (mode === 'create' && props.title === t('pages.admin.servers.tabs.general.page.card.variables', {}))
    return options;
  if (mode === 'update' && props.title === t('pages.admin.servers.tabs.general.page.card.featureLimits', {}))
    return (
      <>
        <OriginalTitleCard {...props} />
        {options}
      </>
    );
  return <OriginalTitleCard {...props} />;
}

export default makeComponentHookable(ConfigurationCard);
