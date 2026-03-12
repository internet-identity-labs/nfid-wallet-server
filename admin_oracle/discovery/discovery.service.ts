import { DiscoveryClient } from "./discovery.client";
import { DiscoveryMapper } from "./discovery.mapper";
import type {
  DiscoveryApp,
  DiscoveryAppResponse,
  DiscoveryService,
} from "./types";

export class DefaultDiscoveryService implements DiscoveryService {
  constructor(
    private readonly client: DiscoveryClient,
    private readonly mapper: DiscoveryMapper,
  ) {}

  getApps(request: Array<DiscoveryApp>): Promise<Array<DiscoveryAppResponse>> {
    return Promise.all(request.map((app) => this.enrichApp(app)));
  }

  private async enrichApp(app: DiscoveryApp): Promise<DiscoveryAppResponse> {
    try {
      const html = await this.client.fetchPage(app.hostname);
      const meta = await this.mapper.toMeta(html, app.hostname);
      return { isError: false, data: { ...app, ...meta } };
    } catch (err) {
      const cause = err instanceof Error ? err.message : String(err);
      return {
        isError: true,
        error: {
          error: `Failed to fetch ${app.hostname}: ${cause}`,
          request: app,
        },
      };
    }
  }
}
