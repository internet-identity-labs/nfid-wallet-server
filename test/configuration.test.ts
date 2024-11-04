import "mocha";
import {expect} from "chai";
import {
    AccessPointRequest,
    BoolHttpResponse,
    ConfigurationRequest,
    ConfigurationResponse,
    HTTPAccountRequest,
} from "./idl/identity_manager";
import {deploy, getActor, getIdentity} from "./util/deployment.util";
import {Dfx} from "./type/dfx";
import {App} from "./constanst/app.enum";
import {DFX} from "./constanst/dfx.const";
import {idlFactory as imIdl} from "./idl/identity_manager_idl";
import {fail} from "assert";

describe("Configuration", () => {
    var dfx: Dfx;

    before(async () => {
        dfx = await deploy({apps: [App.IdentityManager]});
    });

    it("should respond with correct configuration", async function () {
        const result: ConfigurationResponse = (await dfx.im.actor.get_config()) as ConfigurationResponse;
        expect(result.env[0]).to.be.eq("test");
        expect(result.whitelisted_phone_numbers[0]).to.be.an("array").that.is.empty;
        expect(result.backup_canister_id).to.be.an("array").that.is.empty;
        expect(result.ii_canister_id[0]?.toText()).to.be.equal("rdmx6-jaaaa-aaaaa-aaadq-cai");
        expect(result.whitelisted_canisters).to.be.an("array").that.is.empty;
        expect(result.git_branch).to.be.an("array").that.is.empty;
        expect(result.lambda[0]?.toText()).to.be.equal("3ekng-5nqql-esu4u-64sla-pcm5o-hjatn-hwjo7-vk7ya-ianug-zqqyy-iae");
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
            'commit_hash': [],
            'operator': [],
            'account_creation_paused': [],
            'lambda_url': []
        } as ConfigurationRequest;
        const configureResult = await dfx.im.actor.configure(request);
        expect(configureResult).to.be.undefined;

        const configureResponse: ConfigurationResponse = (await dfx.im.actor.get_config()) as ConfigurationResponse;
        expect(configureResponse.env[0]).to.be.eq("dev");
        expect(configureResponse.whitelisted_phone_numbers[0]).to.be.an("array").that.is.empty;
        expect(configureResponse.backup_canister_id).to.be.an("array").that.is.empty;
        expect(configureResponse.ii_canister_id[0]?.toText()).to.be.equal("rdmx6-jaaaa-aaaaa-aaadq-cai");
        expect(configureResponse.whitelisted_canisters).to.be.an("array").that.is.empty;
        expect(configureResponse.git_branch).to.be.an("array").that.is.empty;
        expect(configureResponse.lambda[0]?.toText()).to.be.equal("ritih-icnvs-i7b67-sc2vs-nwo2e-bvpe5-viznv-uqluj-xzcvs-6iqsp-fqe");
        expect(configureResponse.token_refresh_ttl[0]).to.be.equal(60n);
        expect(configureResponse.heartbeat).to.be.an("array").that.is.empty;
        expect(configureResponse.token_ttl[0]).to.be.equal(60n);
        expect(configureResponse.commit_hash).to.be.an("array").that.is.empty;

    });

    it("should keep configuration in stable memory between redeployments", async function () {
        const request = {
            'env': ["dev2"],
            'whitelisted_phone_numbers': [],
            'backup_canister_id': [],
            'ii_canister_id': [],
            'whitelisted_canisters': [],
            'git_branch': [],
            'lambda': [dfx.user.identity.getPrincipal()],
            'token_refresh_ttl': [],
            'heartbeat': [],
            'token_ttl': [],
            'commit_hash': [],
            'operator': [dfx.user.identity.getPrincipal()],
            'account_creation_paused': [],
            'lambda_url': []
        } as ConfigurationRequest;
        const configureResult = await dfx.im.actor.configure(request);
        expect(configureResult).to.be.undefined;

        DFX.UPGRADE_FORCE("identity_manager")

        const configureResponse: ConfigurationResponse = (await dfx.im.actor.get_config()) as ConfigurationResponse;
        expect(configureResponse.env[0]).to.be.eq("dev2");
    });

    it("should block/unblock account creation", async function () {
        await dfx.im.actor.pause_account_creation(true);
        const identity = getIdentity("87654321876543218765432187654311");
        const principal = identity.getPrincipal().toText();
        const dd: AccessPointRequest = {
            icon: "Icon",
            device: "Global",
            pub_key: principal,
            browser: "Browser",
            device_type: {
                Email: null,
            },
            credential_id: [],
        };
        var accountRequest: HTTPAccountRequest = {
            access_point: [dd],
            wallet: [{NFID: null}],
            anchor: 0n,
            email: ["test@test.test"],
        };
        const actor = await getActor(dfx.im.id, identity, imIdl);

        let email_response = await dfx.im.actor.add_email_and_principal_for_create_account_validation("test@test.test", principal, 25) as BoolHttpResponse;

        expect(email_response.status_code).eq(200);
        try {
            await actor.create_account(
                accountRequest
            )
            fail("Should throw an error");
        } catch (e) {
            expect(e.message).to.contain("Account creation is paused due to high demand. Please try again later.");
        }
        await dfx.im.actor.pause_account_creation(false);

        try {
            await actor.create_account(
                accountRequest
            )
        } catch (e) {
            fail("Should not throw an error");
        }

    });

});
