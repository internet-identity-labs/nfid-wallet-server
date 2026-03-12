import { DiscoveryClient } from "./discovery.client";
import { DiscoveryMapper } from "./discovery.mapper";
import { DefaultDiscoveryService } from "./discovery.service";

const discoveryClient = new DiscoveryClient();
const discoveryMapper = new DiscoveryMapper();

export const discoveryService = new DefaultDiscoveryService(
  discoveryClient,
  discoveryMapper,
);
