import metascraper from "metascraper";
import metascraperTitle from "metascraper-title";
import metascraperDescription from "metascraper-description";
import metascraperImage from "metascraper-image";
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
      metascraperImage(),
      metascraperUrl(),
    ]);
  }

  getApps(request: Array<DiscoveryApp>): Promise<Array<DiscoveryAppResponse>> {
    return Promise.all(request.map((app) => this.enrichApp(app)));
  }

  private async fetchPage(url: string): Promise<string> {
    // Identify as a real browser so Cloudflare and similar gateways serve
    // the actual page instead of an anti-bot challenge.
    const response = await fetch(url, {
      headers: {
        "User-Agent":
          "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
        Accept:
          "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
        "Accept-Language": "en-US,en;q=0.9",
      },
    });
    return response.text();
  }

  private async toMeta(html: string, url: string): Promise<HtmlMeta> {
    const meta = await this.scraper({ html, url });
    return {
      url: meta.url ?? undefined,
      name: meta.title ?? undefined,
      image: meta.image ?? undefined,
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
