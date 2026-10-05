import OriginalTerminal from '@/pages/server/console/terminal/Console.tsx';
import { useServerStore } from '@/stores/server.ts';
import { metadataSchema } from './api.ts';
import NativeTerminal from './Console.tsx';

const Terminal: typeof OriginalTerminal = (props) => {
  const server = useServerStore((state) => state.server);
  return metadataSchema.parse(server).incusInstance ? <NativeTerminal {...props} /> : <OriginalTerminal {...props} />;
};

export default Terminal;
