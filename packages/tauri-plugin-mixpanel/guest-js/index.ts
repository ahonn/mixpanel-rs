import { invoke } from '@tauri-apps/api/core'

interface InvokeError {
  code: string;
  detail: string;
}

export class MixpanelError extends Error {
  constructor(
    public code: string,
    public detail: string,
  ) {
    super(`Mixpanel error: ${code} - ${detail}`);
    this.name = 'MixpanelError';
  }
}

function isInvokeError(err: any): err is InvokeError {
  return typeof err === 'object' && err?.hasOwnProperty('code');
}

/**
 * Track an event with optional properties
 */
export async function track(eventName: string, properties?: Record<string, any>): Promise<void> {
  try {
    await invoke('plugin:mixpanel|track', {
      eventName,
      properties: properties ? properties : null,
    });
  } catch (err) {
    if (isInvokeError(err)) {
      const { code, detail } = err;
      throw new MixpanelError(code, detail);
    }
    throw new MixpanelError('ERROR', (err as Error).message);
  }
}

/**
 * Identify a user with a distinct ID and optional properties
 */
export async function identify(distinctId: string, properties?: Record<string, any>): Promise<void> {
  try {
    await invoke('plugin:mixpanel|identify', {
      distinctId,
      properties: properties ? properties : null,
    });
  } catch (err) {
    if (isInvokeError(err)) {
      const { code, detail } = err;
      throw new MixpanelError(code, detail);
    }
    throw new MixpanelError('ERROR', (err as Error).message);
  }
}

/**
 * Create an alias for a distinct ID
 */
export async function alias(distinctId: string, alias: string): Promise<void> {
  try {
    await invoke('plugin:mixpanel|alias', {
      distinctId,
      alias,
    });
  } catch (err) {
    if (isInvokeError(err)) {
      const { code, detail } = err;
      throw new MixpanelError(code, detail);
    }
    throw new MixpanelError('ERROR', (err as Error).message);
  }
}

/**
 * Opt in to Mixpanel tracking
 */
export async function optIn(): Promise<void> {
  try {
    await invoke('plugin:mixpanel|opt_in');
  } catch (err) {
    if (isInvokeError(err)) {
      const { code, detail } = err;
      throw new MixpanelError(code, detail);
    }
    throw new MixpanelError('ERROR', (err as Error).message);
  }
}

/**
 * Opt out from Mixpanel tracking
 */
export async function optOut(): Promise<void> {
  try {
    await invoke('plugin:mixpanel|opt_out');
  } catch (err) {
    if (isInvokeError(err)) {
      const { code, detail } = err;
      throw new MixpanelError(code, detail);
    }
    throw new MixpanelError('ERROR', (err as Error).message);
  }
}

/**
 * Check if user has opted out from Mixpanel tracking
 */
export async function isOptedOut(): Promise<boolean> {
  try {
    return await invoke<boolean>('plugin:mixpanel|is_opted_out');
  } catch (err) {
    if (isInvokeError(err)) {
      const { code, detail } = err;
      throw new MixpanelError(code, detail);
    }
    throw new MixpanelError('ERROR', (err as Error).message);
  }
}
