import { Actor, ActorMethod, Agent, HttpAgent, Identity } from "@dfinity/agent";
import { Ed25519KeyIdentity } from "@dfinity/identity";
import { spawnSync, StdioOptions } from "child_process";
import { readFileSync } from "fs";
import { Configuration } from "../types/configuration";
import { idlFactory } from "../idls/idl";

const path: string = "identity-manager-itest/src/test/resources";
const localhost: string = "http://127.0.0.1:8000";

export const execFile = (file: string, stdio?: StdioOptions, ...params: string[]): string => {
    console.log("> " + [file]);

    var command: string = readFileSync(path + file).toString();

    params.forEach(el => {
        command = command.replace("%s", el);
    });

    return exec(command, stdio)?.replace("\n", "");
};

export const exec = (command: string, stdio?: StdioOptions): string => {
    console.log("> " + [command]);

    var result = spawnSync(command, {
        stdio: stdio || "inherit",
        shell: true,
        encoding: 'utf-8'
    });

    return result.stdout;
};

export const deploy = async (): Promise<Configuration> => {
    var i = 0;
    var rootIdentity: string;
    var imCanisterId: string;
    var exitCode: string;

    const identity = getIdentity();
    const principal = identity.getPrincipal().toString();

    do {
        execFile("/common/dfx_stop");
        exec("rm -rf .dfx");
        execFile("/common/create_test_persona");
        execFile("/common/use_admin_persona");
        rootIdentity = execFile("/common/get_principal", "pipe");
        execFile("/common/init_dfx_project_full");
        execFile("/common/deploy_im");

        imCanisterId = execFile("/common/get_canister_id", "pipe", "identity_manager");

        console.log(">> ", imCanisterId);

        execFile("/common/add_controller", "inherit", imCanisterId, "identity_manager");
        execFile("/common/add_controller", "inherit", principal, "identity_manager");
        exec("dfx canister call identity_manager configure '(record {env = opt \"test\"})'");
        
        exitCode = exec("echo $?", "pipe");
        console.log(">> ", exitCode);
        if(++i >= 5) {
            execFile("/common/dfx_stop");
            console.error("Stopping ApplicationITest");
            process.exit(1);
        }
    } while(exitCode === "0");

    const actor = await getActor(imCanisterId, identity);

    return {rootIdentity, principal, actor};
};

const getIdentity = (): Identity => {
    const seed = new Array(32).fill(0);
    seed[0] = "seed";
    return Ed25519KeyIdentity.generate(new Uint8Array(seed));
};

const getActor = async (imCanisterId: string, identity: Identity): Promise<Record<string, ActorMethod>> => {
    const agent: Agent = new HttpAgent({ host: localhost, identity: identity });
    await agent.fetchRootKey();
    return await Actor.createActor(idlFactory, {agent, canisterId: imCanisterId});
};