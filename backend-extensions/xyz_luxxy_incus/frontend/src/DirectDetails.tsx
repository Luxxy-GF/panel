import { createContext, useContext } from 'react';
import OriginalStatCard from '@/elements/data-display/StatCard.tsx';
import OriginalServerDetails from '@/pages/server/console/stats/ServerDetails.tsx';
import { useTranslations } from '@/providers/TranslationProvider.tsx';
import { useServerStore } from '@/stores/server.ts';
import { metadataSchema } from './api.ts';

const DirectAddress = createContext<string | null>(null);

const DirectDetails: typeof OriginalServerDetails = () => {
  const server = useServerStore((state) => state.server);
  const address = metadataSchema.parse(server).incusAddress;
  return (
    <DirectAddress.Provider value={address}>
      <OriginalServerDetails />
    </DirectAddress.Provider>
  );
};

export const DirectStatCard: typeof OriginalStatCard = (props) => {
  const address = useContext(DirectAddress);
  const { t } = useTranslations();
  if (address && props.label === t('pages.server.console.details.address', {}))
    return <OriginalStatCard {...props} value={address} copyOnClick redact />;
  if (address && props.label === t('pages.server.console.details.port', {})) return <></>;
  return <OriginalStatCard {...props} />;
};

export default DirectDetails;
