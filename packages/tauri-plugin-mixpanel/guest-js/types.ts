export type Dict = Record<string, any>;

export interface PersistenceOptions {
  /**
   * Whether to use persistent storage.
   * @default true
   */
  persistent?: boolean;
}

// Options specific to register/register_once methods
export interface RegisterOptions extends PersistenceOptions {
  /**
   * Number of days since the user's last visit to store the super properties.
   * Only valid for persisted props.
   */
  days?: number;
}

// Represents the Mixpanel configuration related to persistence (subset)
export interface Persistence {
  persistence?: "cookie" | "localStorage";
  persistence_name?: string;
  cookie_domain?: string;
  cookie_expiration?: number;
  cross_site_cookie?: boolean;
  cross_subdomain_cookie?: boolean;
  secure_cookie?: boolean;
  // Add other relevant config properties if needed
}

// Interface for the Mixpanel People API
export interface People {
  set(prop: string | Dict, to?: any): Promise<void>;
  set_once(prop: string | Dict, to?: any): Promise<void>;
  unset(prop: string | string[]): Promise<void>;
  increment(prop: string | Dict, by?: number): Promise<void>;
  append(list_name: string | Dict, value?: any): Promise<void>;
  remove(list_name: string | Dict, value?: any): Promise<void>;
  union(list_name: string | Dict, values?: any): Promise<void>;
  delete_user(): Promise<void>;
}

// Interface for the core Mixpanel API exposed by the plugin
export interface Mixpanel {
  people: People;

  identify(unique_id: string): Promise<void>;
  alias(alias: string, original?: string): Promise<void>;
  track(event_name: string, properties?: Dict): Promise<void>;
  register(properties: Dict, options?: RegisterOptions): Promise<void>;
  register_once(
    properties: Dict,
    defaultValue?: any,
    options?: RegisterOptions,
  ): Promise<void>;
  unregister(property: string, options?: PersistenceOptions): Promise<void>;
  get_distinct_id(): Promise<string | null>;
  get_property(property_name: string): Promise<any | undefined>;
  reset(): Promise<void>;
  time_event(event_name: string): Promise<void>;

  set_group(
    group_key: string,
    group_ids: string | string[] | number | number[],
    options?: PersistenceOptions,
  ): Promise<void>;
  add_group(
    group_key: string,
    group_id: string | number,
    options?: PersistenceOptions,
  ): Promise<void>;
  remove_group(
    group_key: string,
    group_id: string | number,
    options?: PersistenceOptions,
  ): Promise<void>;
}
