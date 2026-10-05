import { useContext } from 'react';
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
  const { t } = useTranslations();
  if (!instance) return <OriginalTitleCard {...props} />;
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
