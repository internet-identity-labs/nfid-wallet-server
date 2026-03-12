import metascraper from "metascraper";
import metascraperTitle from "metascraper-title";
import metascraperDescription from "metascraper-description";
import metascraperFavicon from "metascraper-logo-favicon";
import metascraperUrl from "metascraper-url";
import type { HtmlMeta } from "./types";

export class DiscoveryMapper {
  private readonly scraper = metascraper([
    metascraperTitle(),
    metascraperDescription(),
    metascraperFavicon({ google: false }),
    metascraperUrl(),
  ]);

  async toMeta(html: string, url: string): Promise<HtmlMeta> {
    const meta = await this.scraper({ html, url });
    return {
      url: meta.url ?? undefined,
      name: meta.title ?? undefined,
      icon: meta.logo ?? undefined,
      desc: meta.description ?? undefined,
    };
  }
}
