import "mocha";
import { expect } from "chai";
import { ConfigurationRequest, ConfigurationResponse } from "./idls/identity_manager";
import { deploy, execFile } from "./utils/exec.util";
import { Configuration } from "./types/configuration";

describe("configuration", () => {
    var configuration: Configuration;

    before(async () => {
        configuration = await deploy();
    });

    after(() => {
        execFile("/common/dfx_stop");
    });

    it("should respond with correct configuration", async function () {
        const result: ConfigurationResponse = (await configuration.actor.get_config()) as ConfigurationResponse;
        expect(result.env[0]).to.be.eq("test");
        expect(result.whitelisted_phone_numbers[0]).to.be.an("array").that.is.empty;
        expect(result.backup_canister_id).to.be.an("array").that.is.empty;
        expect(result.ii_canister_id[0]?.toText()).to.be.equal("rdmx6-jaaaa-aaaaa-aaadq-cai");
        expect(result.whitelisted_canisters[0].map(x => x.toString())).contain(configuration.principal);
        expect(result.git_branch).to.be.an("array").that.is.empty;
        expect(result.lambda[0]?.toText()).to.be.equal("25uuv-mb7qi-uxovp-ucbzz-ddung-opxmb-ip4j2-nzrnk-sec53-jusbp-bae");
        expect(result.token_refresh_ttl[0]).to.be.equal(60n);
        expect(result.heartbeat).to.be.an("array").that.is.empty;
        expect(result.token_ttl[0]).to.be.equal(60n);
        expect(result.commit_hash).to.be.an("array").that.is.empty;
    });

    it("should update env field in the configuration", async function () {
        const request = {
            'env': ["dev"],
            'whitelisted_phone_numbers': [],
            'backup_canister_id': [],
            'ii_canister_id': [],
            'whitelisted_canisters': [],
            'git_branch': [],
            'lambda': [],
            'token_refresh_ttl': [],
            'heartbeat': [],
            'token_ttl': [],
            'commit_hash': []
        } as ConfigurationRequest;
        const configureResult = await configuration.actor.configure(request);
        expect(configureResult).to.be.undefined;

        const configureResponse: ConfigurationResponse = (await configuration.actor.get_config()) as ConfigurationResponse;
        expect(configureResponse.env[0]).to.be.eq("dev");
        expect(configureResponse.whitelisted_phone_numbers[0]).to.be.an("array").that.is.empty;
        expect(configureResponse.backup_canister_id).to.be.an("array").that.is.empty;
        expect(configureResponse.ii_canister_id[0]?.toText()).to.be.equal("rdmx6-jaaaa-aaaaa-aaadq-cai");
        expect(configureResponse.whitelisted_canisters[0].map(x => x.toString())).contain(configuration.principal);
        expect(configureResponse.git_branch).to.be.an("array").that.is.empty;
        expect(configureResponse.lambda[0]?.toText()).to.be.equal("25uuv-mb7qi-uxovp-ucbzz-ddung-opxmb-ip4j2-nzrnk-sec53-jusbp-bae");
        expect(configureResponse.token_refresh_ttl[0]).to.be.equal(60n);
        expect(configureResponse.heartbeat).to.be.an("array").that.is.empty;
        expect(configureResponse.token_ttl[0]).to.be.equal(60n);
        expect(configureResponse.commit_hash).to.be.an("array").that.is.empty;
    });
    
});
