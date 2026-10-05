import { type ComponentProps, useContext } from 'react';
import { makeComponentHookable } from 'shared';
import OriginalConfirmation from '@/elements/modals/ConfirmationModal.tsx';
import { useTranslations } from '@/providers/TranslationProvider.tsx';
import { ConfigurationContext } from './ConfigurationScope.tsx';

const PoolConfirmation = (props: ComponentProps<typeof OriginalConfirmation>) => {
  const scope = useContext(ConfigurationContext);
  const { t } = useTranslations();
  if (
    scope?.mode === 'create' &&
    scope.form?.getValues().incusNetworkRequest &&
    props.title === t('pages.admin.servers.tabs.general.page.modal.confirmNoAllocation.title', {})
  )
    return (
      <OriginalConfirmation
        {...props}
        title='Create instance with IP pool'
        confirm='Create instance'
        confirmColor='blue'
      >
        An address will be reserved from the selected pool. Port allocations are not required for this instance.
      </OriginalConfirmation>
    );
  return <OriginalConfirmation {...props} />;
};

export default makeComponentHookable(PoolConfirmation);
