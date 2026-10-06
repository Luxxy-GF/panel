import { createContext } from 'react';
import OriginalServerDetails from '@/pages/server/console/stats/ServerDetails.tsx';
import { useServerStore } from '@/stores/server.ts';
import { metadataSchema } from './api.ts';

export const DirectAddress = createContext<string | null>(null);

const DirectDetails: typeof OriginalServerDetails = () => {
  const server = useServerStore((state) => state.server);
  const address = metadataSchema.parse(server).incusAddress;
  return (
    <DirectAddress.Provider value={address}>
      <OriginalServerDetails />
    </DirectAddress.Provider>
  );
};

export default DirectDetails;
