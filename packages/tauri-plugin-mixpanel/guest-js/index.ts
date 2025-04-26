import { invoke } from "@tauri-apps/api/core";
import type {
  Dict,
  PersistenceOptions,
  RegisterOptions,
  People,
} from "./types";

interface InvokeError {
  code: string;
  detail: string;
}

export class MixpanelError extends Error {
  constructor(public message: string) {
    super(message);
    this.name = "MixpanelError";
  }
}

function isInvokeError(err: any): err is InvokeError {
  console.log(err);
  return typeof err === "object" && err?.hasOwnProperty("code");
}

const people: People = {
  async set(prop: string | Dict, to?: any): Promise<void> {
    try {
      await invoke("plugin:mixpanel|people_set", { prop, to });
    } catch (err) {
      if (isInvokeError(err)) {
        console.error(err);
        throw new MixpanelError(err.detail);
      }
      throw new MixpanelError((err as Error).message);
    }
  },

  async set_once(prop: string | Dict, to?: any): Promise<void> {
    try {
      await invoke("plugin:mixpanel|people_set_once", { prop, to });
    } catch (err) {
      if (isInvokeError(err)) {
        console.error(err);
        throw new MixpanelError(err.detail);
      }
      throw new MixpanelError((err as Error).message);
    }
  },

  async unset(prop: string | string[]): Promise<void> {
    try {
      await invoke("plugin:mixpanel|people_unset", { prop });
    } catch (err) {
      if (isInvokeError(err)) {
        console.error(err);
        throw new MixpanelError(err.detail);
      }
      throw new MixpanelError((err as Error).message);
    }
  },

  async increment(prop: string | Dict, by?: number): Promise<void> {
    try {
      await invoke("plugin:mixpanel|people_increment", { prop, by });
    } catch (err) {
      if (isInvokeError(err)) {
        console.error(err);
        throw new MixpanelError(err.detail);
      }
      throw new MixpanelError((err as Error).message);
    }
  },

  async append(list_name: string | Dict, value?: any): Promise<void> {
    try {
      await invoke("plugin:mixpanel|people_append", {
        listName: list_name,
        value,
      });
    } catch (err) {
      if (isInvokeError(err)) {
        console.error(err);
        throw new MixpanelError(err.detail);
      }
      throw new MixpanelError((err as Error).message);
    }
  },

  async remove(list_name: string | Dict, value?: any): Promise<void> {
    try {
      await invoke("plugin:mixpanel|people_remove", {
        listName: list_name,
        value,
      });
    } catch (err) {
      if (isInvokeError(err)) {
        console.error(err);
        throw new MixpanelError(err.detail);
      }
      throw new MixpanelError((err as Error).message);
    }
  },

  async union(list_name: string | Dict, values?: any): Promise<void> {
    try {
      await invoke("plugin:mixpanel|people_union", {
        listName: list_name,
        values,
      });
    } catch (err) {
      if (isInvokeError(err)) {
        console.error(err);
        throw new MixpanelError(err.detail);
      }
      throw new MixpanelError((err as Error).message);
    }
  },

  async delete_user(): Promise<void> {
    try {
      await invoke("plugin:mixpanel|people_delete_user");
    } catch (err) {
      if (isInvokeError(err)) {
        console.error(err);
        throw new MixpanelError(err.detail);
      }
      throw new MixpanelError((err as Error).message);
    }
  },
};

const mixpanel = {
  people,

  async register(properties: Dict, options?: RegisterOptions): Promise<void> {
    try {
      await invoke("plugin:mixpanel|register", {
        properties,
        options: options || null,
      });
    } catch (err) {
      if (isInvokeError(err)) {
        console.error(err);
        throw new MixpanelError(err.detail);
      }
      throw new MixpanelError((err as Error).message);
    }
  },

  async register_once(
    properties: Dict,
    defaultValue?: any,
    options?: RegisterOptions,
  ): Promise<void> {
    try {
      await invoke("plugin:mixpanel|register_once", {
        properties,
        defaultValue: defaultValue === undefined ? null : defaultValue,
        options: options || null,
      });
    } catch (err) {
      if (isInvokeError(err)) {
        console.error(err);
        throw new MixpanelError(err.detail);
      }
      throw new MixpanelError((err as Error).message);
    }
  },

  async unregister(
    property: string,
    options?: PersistenceOptions,
  ): Promise<void> {
    try {
      await invoke("plugin:mixpanel|unregister", {
        propertyName: property,
        options: options || null,
      });
    } catch (err) {
      if (isInvokeError(err)) {
        console.error(err);
        throw new MixpanelError(err.detail);
      }
      throw new MixpanelError((err as Error).message);
    }
  },

  async identify(unique_id: string): Promise<void> {
    try {
      await invoke("plugin:mixpanel|identify", { distinctId: unique_id });
    } catch (err) {
      if (isInvokeError(err)) {
        console.error(err);
        throw new MixpanelError(err.detail);
      }
      throw new MixpanelError((err as Error).message);
    }
  },

  async alias(alias: string, original?: string): Promise<void> {
    try {
      await invoke("plugin:mixpanel|alias", { alias, original });
    } catch (err) {
      if (isInvokeError(err)) {
        console.error(err);
        throw new MixpanelError(err.detail);
      }
      throw new MixpanelError((err as Error).message);
    }
  },

  async track(event_name: string, properties?: Dict): Promise<void> {
    console.log("Tracking event:", event_name, properties);
    try {
      await invoke("plugin:mixpanel|track", {
        eventName: event_name,
        properties: properties || {},
      });
    } catch (err) {
      if (isInvokeError(err)) {
        console.error(err);
        throw new MixpanelError(err.detail);
      }
      throw new MixpanelError((err as Error).message);
    }
  },

  async get_distinct_id(): Promise<string | null> {
    try {
      return await invoke("plugin:mixpanel|get_distinct_id");
    } catch (err) {
      if (isInvokeError(err)) {
        console.error(err);
        throw new MixpanelError(err.detail);
      }
      throw new MixpanelError((err as Error).message);
    }
  },

  async get_property(property_name: string): Promise<any | undefined> {
    try {
      return await invoke("plugin:mixpanel|get_property", {
        propertyName: property_name,
      });
    } catch (err) {
      if (isInvokeError(err)) {
        console.error(err);
        throw new MixpanelError(err.detail);
      }
      throw new MixpanelError((err as Error).message);
    }
  },

  async reset(): Promise<void> {
    try {
      await invoke("plugin:mixpanel|reset");
    } catch (err) {
      if (isInvokeError(err)) {
        console.error(err);
        throw new MixpanelError(err.detail);
      }
      throw new MixpanelError((err as Error).message);
    }
  },

  async time_event(event_name: string): Promise<void> {
    try {
      await invoke("plugin:mixpanel|time_event", { eventName: event_name });
    } catch (err) {
      if (isInvokeError(err)) {
        console.error(err);
        throw new MixpanelError(err.detail);
      }
      throw new MixpanelError((err as Error).message);
    }
  },

  async set_group(
    group_key: string,
    group_ids: string | string[] | number | number[],
    options?: PersistenceOptions,
  ): Promise<void> {
    try {
      await invoke("plugin:mixpanel|set_group", {
        groupKey: group_key,
        groupIds: group_ids,
        options: options || null,
      });
    } catch (err) {
      if (isInvokeError(err)) {
        console.error(err);
        throw new MixpanelError(err.detail);
      }
      throw new MixpanelError((err as Error).message);
    }
  },

  async add_group(
    group_key: string,
    group_id: string | number,
    options?: PersistenceOptions,
  ): Promise<void> {
    try {
      await invoke("plugin:mixpanel|add_group", {
        groupKey: group_key,
        groupId: group_id,
        options: options || null,
      });
    } catch (err) {
      if (isInvokeError(err)) {
        console.error(err);
        throw new MixpanelError(err.detail);
      }
      throw new MixpanelError((err as Error).message);
    }
  },

  async remove_group(
    group_key: string,
    group_id: string | number,
    options?: PersistenceOptions,
  ): Promise<void> {
    try {
      await invoke("plugin:mixpanel|remove_group", {
        groupKey: group_key,
        groupId: group_id,
        options: options || null,
      });
    } catch (err) {
      if (isInvokeError(err)) {
        console.error(err);
        throw new MixpanelError(err.detail);
      }
      throw new MixpanelError((err as Error).message);
    }
  },
};

export default mixpanel;
