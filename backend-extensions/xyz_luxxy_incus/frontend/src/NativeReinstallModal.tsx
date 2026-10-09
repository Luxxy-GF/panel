import { ModalProps } from '@mantine/core';
import { useNavigate } from 'react-router';
import { z } from 'zod';
import installServer from '@/api/server/settings/installServer.ts';
import Button from '@/elements/buttons/Button.tsx';
import Alert from '@/elements/feedback/Alert.tsx';
import Switch from '@/elements/input/Switch.tsx';
import Stack from '@/elements/layout/Stack.tsx';
import FormModal from '@/elements/modals/FormModal.tsx';
import { ModalFooter } from '@/elements/modals/Modal.tsx';
import { serverSettingsReinstallSchema } from '@/lib/schemas/server/settings.ts';
import OriginalModal from '@/pages/server/settings/modals/SettingsReinstallModal.tsx';
import { useModalForm } from '@/plugins/form/useModalForm.ts';
import { useServerCan } from '@/plugins/usePermissions.ts';
import { useToast } from '@/providers/ToastProvider.tsx';
import { useTranslations } from '@/providers/TranslationProvider.tsx';
import { useServerStore } from '@/stores/server.ts';
import { metadataSchema } from './api.ts';

export default function NativeReinstallModal(props: ModalProps) {
  const server = useServerStore((state) => state.server);
  return metadataSchema.parse(server).incusInstance ? <RecreateModal {...props} /> : <OriginalModal {...props} />;
}

function RecreateModal({ ...props }: ModalProps) {
  const { t } = useTranslations();
  const { addToast } = useToast();
  const server = useServerStore((state) => state.server);
  const updateServer = useServerStore((state) => state.updateServer);
  const navigate = useNavigate();
  const canStart = useServerCan('control.start');

  const { form, handleClose, handleSubmit, loading, isDirty } = useModalForm<
    z.infer<typeof serverSettingsReinstallSchema>
  >({
    initialValues: {
      truncateDirectory: false,
      startOnCompletion: false,
    },
    schema: serverSettingsReinstallSchema,
    onClose: props.onClose,
    onSubmit: async (values) => {
      await installServer(server.uuid, values);
      addToast(t('pages.server.settings.reinstall.modal.toast.reinstalling', {}), 'success');
      navigate(`/server/${server.uuidShort}`);
      updateServer({ status: 'installing' });
    },
  });

  return (
    <FormModal
      title='Recreate OS instance'
      isDirty={isDirty}
      loading={loading}
      {...props}
      onClose={handleClose}
      onSubmit={handleSubmit}
    >
      <Stack>
        <Alert>
          The existing OS instance and its entire guest filesystem will be deleted, including installed applications and
          snapshots. A fresh instance will use the selected image, IP address, resource limits and saved Incus
          configuration. This cannot be cancelled once accepted.
        </Alert>
        <Switch
          label='I understand that the guest filesystem will be erased'
          name='truncate'
          {...form.getInputProps('truncateDirectory', { type: 'checkbox' })}
        />
        {canStart && (
          <Switch
            label={t('pages.server.settings.reinstall.modal.startOnCompletion', {})}
            name='startOnCompletion'
            {...form.getInputProps('startOnCompletion', { type: 'checkbox' })}
          />
        )}
      </Stack>

      <ModalFooter>
        <Button
          color='red'
          type='submit'
          loading={loading}
          disabled={!form.isValid() || !form.values.truncateDirectory}
        >
          Recreate instance
        </Button>
        <Button variant='default' onClick={handleClose}>
          {t('common.button.close', {})}
        </Button>
      </ModalFooter>
    </FormModal>
  );
}
