import "mocha";
import { expect } from "chai";
import { Application, BoolHttpResponse, ConfigurationRequest, ConfigurationResponse, HTTPAccountRequest, HTTPAccountResponse, HTTPApplicationResponse, PersonaRequest } from "./idl/identity_manager";
import { deploy } from "./util/deployment.util";
import { Dfx } from "./type/dfx";
import { App } from "./constanst/app.enum";
import { DFX } from "./constanst/dfx.const";

const userLimit = 1;
const appName = 'TEST_APP';
const appName2 = 'TEST_APP2';
const appName3 = 'TEST_PERSONA';
const domain = 'dom';
const domain2 = 'dom2';
const domain3 = 'TEST_DOMAIN';
const personaName = 'PERSONA_NAME';
const personaId = 'PERSONA';

describe("Application", () => {
    var dfx: Dfx;

    before(async () => {
        dfx = await deploy(App.IdentityManager);
    });

    after(() => {
        DFX.STOP();
    });

    it("should create an application.", async function () {
        const application: Application = {
            user_limit: userLimit,
            domain,
            name: appName
        };
        const response: HTTPApplicationResponse = (await dfx.im.actor.create_application(application)) as HTTPApplicationResponse;
        expect(response.status_code).eq(200);
        expect(response.error).an("array").empty;
        expect(response.data[0]).an("array").not.empty;
        expect(response.data[0].length).eq(1);

        const app: Application = response.data[0][0];
        expect(app.user_limit).eq(userLimit);
        expect(app.domain).eq(domain);
        expect(app.name).eq(appName);
    });

    it("should return an error due the same name on creating application.", async function () {
        const application: Application = {
            user_limit: userLimit,
            domain: domain2,
            name: appName
        };
        const response: HTTPApplicationResponse = (await dfx.im.actor.create_application(application)) as HTTPApplicationResponse;
        expect(response.status_code).eq(404);
        expect(response.data).an("array").empty;
        expect(response.error).an("array").not.empty;
        expect(response.error.length).eq(1);
        expect(response.error[0]).eq("Unable to create Application. Application exists");
    });

    it("should create the second application with different name and domain.", async function () {
        const application: Application = {
            user_limit: userLimit,
            domain: domain2,
            name: appName2
        };
        const response: HTTPApplicationResponse = (await dfx.im.actor.create_application(application)) as HTTPApplicationResponse;
        expect(response.status_code).eq(200);
        expect(response.error).an("array").empty;
        expect(response.data[0]).an("array").not.empty;
        expect(response.data[0].length).eq(2);

        const app: Application = response.data[0][0];
        expect(app.user_limit).eq(userLimit);
        expect(app.domain).eq(domain);
        expect(app.name).eq(appName);

        const app2: Application = response.data[0][1];
        expect(app2.user_limit).eq(userLimit);
        expect(app2.domain).eq(domain2);
        expect(app2.name).eq(appName2);
    });

    it("should read the applications.", async function () {
        const response: HTTPApplicationResponse = (await dfx.im.actor.read_applications()) as HTTPApplicationResponse;
        expect(response.status_code).eq(200);
        expect(response.error).an("array").empty;
        expect(response.data[0]).an("array").not.empty;
        expect(response.data[0].length).eq(2);

        const app: Application = response.data[0][0];
        expect(app.user_limit).eq(userLimit);
        expect(app.domain).eq(domain);
        expect(app.name).eq(appName);

        const app2: Application = response.data[0][1];
        expect(app2.user_limit).eq(userLimit);
        expect(app2.domain).eq(domain2);
        expect(app2.name).eq(appName2);
    });

    it("should respond with over the limit response.", async function () {
        const accountRequest: HTTPAccountRequest = {
            anchor: 10000n
        };
        const persona1: PersonaRequest = {
            domain: domain3,
            persona_name: personaName,
            persona_id: personaId,
        };
        const persona2: PersonaRequest = {
            domain: 'TEST_DOMAIN_DD',
            persona_name: 'TEST_NAME_DD',
            persona_id: 'TEST_ID_DD',
        };
        const application: Application = {
            user_limit: userLimit,
            domain: domain3,
            name: appName3
        };

        await dfx.im.actor.create_account(accountRequest);
        await dfx.im.actor.create_persona(persona1);
        await dfx.im.actor.create_persona(persona2);

        const negativeResponse: BoolHttpResponse = (await dfx.im.actor.is_over_the_application_limit(domain3)) as BoolHttpResponse;
        expect(negativeResponse.status_code).eq(200);
        expect(negativeResponse.error).an("array").empty;
        expect(negativeResponse.data).an("array").not.empty;
        expect(negativeResponse.data[0]).eq(false);

        await dfx.im.actor.create_application(application);

        const positiveResponse: BoolHttpResponse = (await dfx.im.actor.is_over_the_application_limit(domain3)) as BoolHttpResponse;
        expect(positiveResponse.status_code).eq(200);
        expect(positiveResponse.error).an("array").empty;
        expect(positiveResponse.data).an("array").not.empty;
        expect(positiveResponse.data[0]).eq(true);
    });

    it("should delete an application with correct deletion and over the limit response.", async function () {
        const deleteResponse: BoolHttpResponse = (await dfx.im.actor.delete_application(appName3)) as BoolHttpResponse;
        expect(deleteResponse.status_code).eq(200);
        expect(deleteResponse.error).an("array").empty;
        expect(deleteResponse.data).an("array").not.empty;
        expect(deleteResponse.data[0]).eq(true);

        const limitResponse: BoolHttpResponse = (await dfx.im.actor.is_over_the_application_limit(domain3)) as BoolHttpResponse;
        expect(limitResponse.status_code).eq(200);
        expect(limitResponse.error).an("array").empty;
        expect(limitResponse.data).an("array").not.empty;
        expect(limitResponse.data[0]).eq(false);
    });

    it("should create personas to exceed the limit of the application.", async function () {
        const persona: PersonaRequest = {
            domain: domain3,
            persona_name: personaName,
            persona_id: personaId,
        };
        const application: Application = {
            user_limit: userLimit,
            domain: domain3,
            name: appName3
        };

        await dfx.im.actor.create_persona(persona);
        await dfx.im.actor.create_persona(persona);
        await dfx.im.actor.create_persona(persona);
        await dfx.im.actor.create_persona(persona);
        await dfx.im.actor.create_persona(persona);

        const personaResponse: HTTPAccountResponse = (await dfx.im.actor.create_persona(persona)) as HTTPAccountResponse;
        expect(personaResponse.status_code).eq(404);
        expect(personaResponse.data).an("array").empty;
        expect(personaResponse.error).an("array").not.empty;
        expect(personaResponse.error.length).eq(1);
        expect(personaResponse.error[0]).eq("Impossible to link this domain. Over limit.");

        await dfx.im.actor.create_application(application);

        const limitResponse: BoolHttpResponse = (await dfx.im.actor.is_over_the_application_limit(domain3)) as BoolHttpResponse;
        expect(limitResponse.status_code).eq(200);
        expect(limitResponse.error).an("array").empty;
        expect(limitResponse.data).an("array").not.empty;
        expect(limitResponse.data[0]).eq(true);
    });
});