import { spawnSync } from "child_process";

export const mochaHooks = {
    beforeAll() {
        spawnSync("dfx start --clean --background", {
            stdio: "inherit",
            shell: true,
            encoding: "utf-8",
        });
    },
    afterAll() {
        spawnSync("dfx stop", {
            stdio: "inherit",
            shell: true,
            encoding: "utf-8",
        });
    },
};
