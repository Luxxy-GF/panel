import OriginalCreate from '@/api/admin/servers/createServer.ts';
import OriginalUpdateApi from '@/api/admin/servers/updateServer.ts';
import OriginalTitleCard from '@/elements/data-display/TitleCard.tsx';
import OriginalCreatePage from '@/pages/admin/servers/ServerCreate.tsx';
import OriginalUpdate from '@/pages/admin/servers/ServerUpdate.tsx';
import OriginalTerminal from '@/pages/server/console/terminal/Console.tsx';
import ConfigurationCard from './ConfigurationCard.tsx';
import Create from './createServer.ts';
import CreatePage from './ServerCreate.tsx';
import Update from './ServerUpdate.tsx';
import Terminal from './Terminal.tsx';
import UpdateApi from './updateServer.ts';

function defineOverride<T>(original: T, replacement: NoInfer<T>) {
  return { original, replacement };
}

export default [
  defineOverride(OriginalCreatePage, CreatePage),
  defineOverride(OriginalTitleCard, ConfigurationCard),
  defineOverride(OriginalUpdateApi, UpdateApi),
  defineOverride(OriginalCreate, Create),
  defineOverride(OriginalUpdate, Update),
  defineOverride(OriginalTerminal, Terminal),
];
