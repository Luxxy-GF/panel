import { useContext } from 'react';
import OriginalStatCard from '@/elements/data-display/StatCard.tsx';
import { useTranslations } from '@/providers/TranslationProvider.tsx';
import { DirectAddress } from './DirectDetails.tsx';

const DirectStatCard: typeof OriginalStatCard = (props) => {
  const address = useContext(DirectAddress);
  const { t } = useTranslations();
  if (address && props.label === t('pages.server.console.details.address', {}))
    return <OriginalStatCard {...props} value={address} copyOnClick redact />;
  if (address && props.label === t('pages.server.console.details.port', {})) return <></>;
  return <OriginalStatCard {...props} />;
};

export default DirectStatCard;
