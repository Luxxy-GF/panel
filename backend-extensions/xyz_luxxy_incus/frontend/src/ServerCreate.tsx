import OriginalServerCreate from '@/pages/admin/servers/ServerCreate.tsx';
import { ConfigurationScope } from './ConfigurationScope.tsx';

const ServerCreate: typeof OriginalServerCreate = () => (
  <ConfigurationScope mode='create'>
    <OriginalServerCreate />
  </ConfigurationScope>
);

export default ServerCreate;
