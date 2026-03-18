export interface DiscoveryApp {
  id: number;
  derivationOrigin?: string;
  hostname: string;
  url?: string;
  name?: string;
  image?: string;
  desc?: string;
  isGlobal: boolean;
  isAnonymous: boolean;
  uniqueUsers: number;
}

export interface DiscoveryAppError {
  error: string;
  request: DiscoveryApp;
}

export type Result<T, E> =
  | { isError: false; data: T }
  | { isError: true; error: E };

export type DiscoveryAppResponse = Result<DiscoveryApp, DiscoveryAppError>;

export interface DiscoveryService {
  getApps: (
    request: Array<DiscoveryApp>,
  ) => Promise<Array<DiscoveryAppResponse>>;
}

export interface HtmlMeta {
  url: string;
  name?: string;
  image?: string;
  desc?: string;
}
