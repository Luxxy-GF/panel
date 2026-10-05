import OriginalServerUpdate from '@/pages/admin/servers/ServerUpdate.tsx';
import { metadataSchema } from './api.ts';
import { EditingInstance } from './InstanceControls.tsx';

const ServerUpdate: typeof OriginalServerUpdate = (props) => (
  <EditingInstance.Provider value={metadataSchema.parse(props.contextServer).incusInstance}>
    <OriginalServerUpdate {...props} />
  </EditingInstance.Provider>
);

export default ServerUpdate;
