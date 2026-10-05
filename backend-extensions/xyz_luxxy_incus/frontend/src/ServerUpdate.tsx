import OriginalServerUpdate from '@/pages/admin/servers/ServerUpdate.tsx';
import { metadataSchema } from './api.ts';
import { ConfigurationScope } from './ConfigurationScope.tsx';
import { EditingInstance } from './InstanceControls.tsx';

const ServerUpdate: typeof OriginalServerUpdate = (props) => (
  <EditingInstance.Provider value={metadataSchema.parse(props.contextServer).incusInstance}>
    <ConfigurationScope mode='update'>
      <OriginalServerUpdate {...props} />
    </ConfigurationScope>
  </EditingInstance.Provider>
);

export default ServerUpdate;
