import OriginalCreate from '@/api/admin/servers/createServer.ts';
import OriginalUpdateApi from '@/api/admin/servers/updateServer.ts';
import OriginalStatCard from '@/elements/data-display/StatCard.tsx';
import OriginalTitleCard from '@/elements/data-display/TitleCard.tsx';
import OriginalConfirmation from '@/elements/modals/ConfirmationModal.tsx';
import OriginalCreatePage from '@/pages/admin/servers/ServerCreate.tsx';
import OriginalUpdate from '@/pages/admin/servers/ServerUpdate.tsx';
import OriginalDetails from '@/pages/server/console/stats/ServerDetails.tsx';
import OriginalTerminal from '@/pages/server/console/terminal/Console.tsx';
import OriginalReinstallModal from '@/pages/server/settings/modals/SettingsReinstallModal.tsx';
import ConfigurationCard from './ConfigurationCard.tsx';
import Create from './createServer.ts';
import DirectDetails from './DirectDetails.tsx';
import DirectStatCard from './DirectStatCard.tsx';
import NativeReinstallModal from './NativeReinstallModal.tsx';
import PoolConfirmation from './PoolConfirmation.tsx';
import CreatePage from './ServerCreate.tsx';
import Update from './ServerUpdate.tsx';
import Terminal from './Terminal.tsx';
import UpdateApi from './updateServer.ts';

function defineOverride<T>(original: T, replacement: NoInfer<T>) {
  return { original, replacement };
}

export default [
  defineOverride(OriginalReinstallModal, NativeReinstallModal),
  defineOverride(OriginalConfirmation, PoolConfirmation),
  defineOverride(OriginalDetails, DirectDetails),
  defineOverride(OriginalStatCard, DirectStatCard),
  defineOverride(OriginalCreatePage, CreatePage),
  defineOverride(OriginalTitleCard, ConfigurationCard),
  defineOverride(OriginalUpdateApi, UpdateApi),
  defineOverride(OriginalCreate, Create),
  defineOverride(OriginalUpdate, Update),
  defineOverride(OriginalTerminal, Terminal),
];
