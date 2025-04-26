export type Dict = Record<string, any>;

export interface PersistenceOptions {
  persistent?: boolean;
}

export interface RegisterOptions extends PersistenceOptions {
  days?: number;
}

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
