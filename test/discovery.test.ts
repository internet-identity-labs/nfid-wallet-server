import "mocha";
import { expect } from "chai";
import { discoveryService } from "../admin_oracle/discovery";
import type { DiscoveryApp } from "../admin_oracle/discovery";

const NFID_APP: DiscoveryApp = {
  id: 1,
  derivationOrigin: "https://3y5ko-7qaaa-aaaal-aaaaq-cai.icp0.io",
  hostname: "https://nfid.one",
  isGlobal: true,
  isAnonymous: true,
  uniqueUsers: 10,
};

const KONG_SWAP_APP: DiscoveryApp = {
  id: 2,
  derivationOrigin: "https://3y5ko-7qaaa-aaaal-aaaaq-cai.icp0.io",
  hostname: "https://kongswap.io",
  isGlobal: true,
  isAnonymous: true,
  uniqueUsers: 2,
};

const UBIN_APP: DiscoveryApp = {
  id: 3,
  derivationOrigin: null,
  hostname: "https://h3cjw-syaaa-aaaam-qbbia-cai.ic0.app",
  isGlobal: false,
  isAnonymous: true,
  uniqueUsers: 20,
};

const ATOMIC_WALLET_APP: DiscoveryApp = {
  id: 4,
  derivationOrigin: null,
  hostname: "https://atomicwallet.io/",
  isGlobal: true,
  isAnonymous: true,
  uniqueUsers: 200,
};

describe("DiscoveryService", () => {
  describe("getApps", () => {
    it("should enrich apps with url, name, image and desc", async () => {
      // Given valid apps with reachable hostnames
      const request = [NFID_APP, KONG_SWAP_APP, UBIN_APP, ATOMIC_WALLET_APP];

      // When getApps is called
      const apps = await discoveryService.getApps(request);

      // Then each result should be enriched with the expected metadata
      expect(apps).to.deep.equal([
        {
          isError: false,
          data: {
            ...NFID_APP,
            url: "https://nfid.one/",
            name: "NFID Wallet | The ICP Wallet",
            image: "https://nfid.one/assets/nfid-wallet-og.png",
            desc: "The easiest to use, hardest to lose, and only wallet governed by a DAO.",
          },
        },
        {
          isError: false,
          data: {
            ...KONG_SWAP_APP,
            url: "https://kongswap.io/",
            name: "KongSwap - Internet Computer’s Leading DEX | Zero Gas Cross-Chain Trading",
            image: "https://kongswap.io/images/og-banner.png",
            desc: "The Internet Computer’s leading DEX for seamless, zero-gas crypto swaps. Swap ICP and SOL tokens with best rates and zero slippage.",
          },
        },
        {
          isError: false,
          data: {
            ...UBIN_APP,
            url: "https://h3cjw-syaaa-aaaam-qbbia-cai.ic0.app",
            name: "Welcome to uBin!",
            image: undefined,
            desc: "The only cloud storage platform where you truly own your data.",
          },
        },
        {
          isError: false,
          data: {
            ...ATOMIC_WALLET_APP,
            url: "https://atomicwallet.io/",
            name: "Best Cryptocurrency Wallet for Mobile & Desktop: Bitcoin & Altcoins",
            image: "https://atomicwallet.io/images/wallets/cryptocurrency_bitcoin_wallet-ios.png",
            desc: "Buy, stake, swap, and manage cryptocurrencies with the best Cryptocurrency Wallet & Bitcoin Wallet. Secure Atomic Wallet for your crypto assets and NFTs.",
          },
        },
      ]);
    });

    it("should return DiscoveryAppError for unreachable hostname", async () => {
      // Given an app with an unreachable hostname
      const badApp: DiscoveryApp = {
        ...NFID_APP,
        id: 2,
        hostname: "https://this-host-does-not-exist.invalid",
      };

      // When getApps is called
      const [result] = await discoveryService.getApps([badApp]);

      // Then the result should be a DiscoveryAppError with the original request
      expect(result).to.deep.equal({
        isError: true,
        error: {
          error: `Failed to fetch ${badApp.hostname}: fetch failed`,
          request: badApp,
        },
      });
    });

    it("should process multiple apps in parallel and return one result per input", async () => {
      // Given a mix of reachable and unreachable apps
      const apps: DiscoveryApp[] = [
        NFID_APP,
        {
          ...NFID_APP,
          id: 2,
          hostname: "https://this-host-does-not-exist.invalid",
        },
      ];

      // When getApps is called
      const results = await discoveryService.getApps(apps);

      // Then each app should have a corresponding result in the same order
      expect(results).to.have.length(2);
      expect(results[0].isError).to.be.false;
      expect(results[1].isError).to.be.true;
    });
  });
});
