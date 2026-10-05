import OriginalCreate from '@/api/admin/servers/createServer.ts';
import OriginalUpdate from '@/pages/admin/servers/ServerUpdate.tsx';
import OriginalTerminal from '@/pages/server/console/terminal/Console.tsx';
import Create from './createServer.ts';
import Update from './ServerUpdate.tsx';
import Terminal from './Terminal.tsx';

function defineOverride<T>(original: T, replacement: NoInfer<T>) {
  return { original, replacement };
}

export default [
  defineOverride(OriginalCreate, Create),
  defineOverride(OriginalUpdate, Update),
  defineOverride(OriginalTerminal, Terminal),
];
