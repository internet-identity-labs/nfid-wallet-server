export class DiscoveryClient {
  async fetchPage(url: string): Promise<string> {
    const response = await fetch(url);
    return response.text();
  }
}
