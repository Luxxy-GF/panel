import { Extension, type ExtensionContext } from 'shared';
import type { FieldDef } from '@/elements/form-engine/index.ts';
import { insertFieldsBefore } from '@/elements/form-engine/index.ts';
import { instanceSchema } from './api.ts';
import InstanceControls, { type InstanceForm } from './InstanceControls.tsx';
import './app.css';

class IncusExtension extends Extension {
  public initialize(ctx: ExtensionContext): void {
    for (const formId of ['admin.servers.create', 'admin.servers.update'] as const) {
      ctx.extensionRegistry.enterForms((forms) =>
        forms.extend(formId, {
          zodShape: { incusInstance: instanceSchema.nullable() },
          initialValues: { incusInstance: null },
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
