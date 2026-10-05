import { faNetworkWired } from '@fortawesome/free-solid-svg-icons';
import { Extension, type ExtensionContext } from 'shared';
import type { FieldDef } from '@/elements/form-engine/index.ts';
import { insertFieldsBefore } from '@/elements/form-engine/index.ts';
import { instanceSchema, networkRequestSchema } from './api.ts';
import InstanceControls, { type InstanceForm } from './InstanceControls.tsx';
import IpPools from './IpPools.tsx';
import './app.css';

class IncusExtension extends Extension {
  public initialize(ctx: ExtensionContext): void {
    ctx.extensionRegistry.pages.admin.nodes.view.subNavigation.addItemInterceptor((items, { node }) => {
      items.push({
        name: 'IP Pools',
        icon: faNetworkWired,
        path: '/incus-ip-pools',
        element: <IpPools node={node} />,
        permission: 'nodes.allocations',
      });
    });
    for (const formId of ['admin.servers.create', 'admin.servers.update'] as const) {
      ctx.extensionRegistry.enterForms((forms) =>
        forms.extend(formId, {
          zodShape: { incusInstance: instanceSchema.nullable(), incusNetworkRequest: networkRequestSchema.nullable() },
          initialValues: { incusInstance: null, incusNetworkRequest: null },
          transform: (fields) => {
            const applicationFields = new Set([
              'image',
              'startup',
              'eggUuid',
              '_nestSelect',
              '_predefinedImage',
              'skipInstaller',
              'hugepagesPassthroughEnabled',
              'kvmPassthroughEnabled',
            ]);
            const adjusted = fields.map((field) =>
              applicationFields.has(field.name)
                ? {
                    ...field,
                    when: (values: Record<string, unknown>) => !values.incusInstance && (field.when?.(values) ?? true),
                  }
                : field,
            );
            return insertFieldsBefore(adjusted, 'image', {
              type: 'custom',
              name: '_incusControls',
              colSpan: 'full',
              render: (form) => (
                <InstanceControls form={form as InstanceForm} editing={formId === 'admin.servers.update'} />
              ),
            } satisfies FieldDef);
          },
        }),
      );
    }
  }
}

export default new IncusExtension();
