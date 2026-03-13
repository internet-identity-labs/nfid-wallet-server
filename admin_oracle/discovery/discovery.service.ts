import metascraper from "metascraper";
import metascraperTitle from "metascraper-title";
import metascraperDescription from "metascraper-description";
import metascraperFavicon from "metascraper-logo-favicon";
import metascraperUrl from "metascraper-url";
import type {
  DiscoveryApp,
  DiscoveryAppResponse,
  DiscoveryService,
  HtmlMeta,
} from "./types";

export class DefaultDiscoveryService implements DiscoveryService {
  private readonly scraper: ReturnType<typeof metascraper>;

  constructor() {
    this.scraper = metascraper([
      metascraperTitle(),
      metascraperDescription(),
      metascraperFavicon({ google: false }),
      metascraperUrl(),
    ]);
  }

  getApps(request: Array<DiscoveryApp>): Promise<Array<DiscoveryAppResponse>> {
    return Promise.all(request.map((app) => this.enrichApp(app)));
  }

  private async fetchPage(url: string): Promise<string> {
    const response = await fetch(url);
    return response.text();
  }

  private async toMeta(html: string, url: string): Promise<HtmlMeta> {
    const meta = await this.scraper({ html, url });
    return {
      url: meta.url ?? undefined,
      name: meta.title ?? undefined,
      icon: meta.logo ?? undefined,
      desc: meta.description ?? undefined,
    };
  }

  private async enrichApp(app: DiscoveryApp): Promise<DiscoveryAppResponse> {
    try {
      const html = await this.fetchPage(app.hostname);
      const meta = await this.toMeta(html, app.hostname);
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

export const discoveryService = new DefaultDiscoveryService();
