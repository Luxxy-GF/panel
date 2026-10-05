import { createContext, type ReactNode, useContext, useEffect, useRef, useState } from 'react';
import type { NativeInstance } from './api.ts';
import type { InstanceForm } from './InstanceControls.tsx';

export const ConfigurationContext = createContext<{
  mode: 'create' | 'update';
  form: InstanceForm | null;
  register: (form: InstanceForm) => void;
} | null>(null);

export function ConfigurationScope({ mode, children }: { mode: 'create' | 'update'; children: ReactNode }) {
  const [form, register] = useState<InstanceForm | null>(null);
  return <ConfigurationContext.Provider value={{ mode, form, register }}>{children}</ConfigurationContext.Provider>;
}

export function useConfigurationForm(form: InstanceForm) {
  const register = useContext(ConfigurationContext)?.register;
  const initialForm = useRef(form);
  useEffect(() => {
    register?.(initialForm.current);
  }, [register]);
}

export function useInstance(form: InstanceForm) {
  const [instance, setInstance] = useState<NativeInstance | null>(null);
  form.watch('incusInstance', ({ value }) => setInstance((value as NativeInstance | null) ?? null));
  useEffect(() => {
    setInstance((form.getValues().incusInstance as NativeInstance | null) ?? null);
  }, [form]);
  return instance;
}
