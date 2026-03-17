import { execute } from "../util/call.util";
import { App, APP_CANISTER_IDS } from "./app.enum";

export class DeployBuilder {
  constructor(
    private app: App,
    private argument?: string,
  ) {}

  withArgument(arg: string): this {
    this.argument = arg;
    return this;
  }

  run(): void {
    const devId = APP_CANISTER_IDS[this.app];
    const idPart = devId ? `--specified-id ${devId}` : "";
    const argPart = this.argument ? `--argument '${this.argument}'` : "";
    execute(
      `dfx deploy ${this.app} --no-wallet ${argPart} ${idPart} --mode reinstall -y`,
    );
  }
}
