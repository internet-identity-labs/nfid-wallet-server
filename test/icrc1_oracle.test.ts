import {Dfx} from "./type/dfx";
import {deploy, getActor, getIdentity} from "./util/deployment.util";
import {App} from "./constanst/app.enum";
import {expect} from "chai";
import {ICRC1, NeuronData, DiscoveryApp, DiscoveryVisitRequest} from "./idl/icrc1_oracle";
import {idlFactory} from "./idl/icrc1_oracle_idl";
import {fail} from "assert";
import {DFX} from "./constanst/dfx.const";

describe("ICRC1 canister Oracle", () => {
    var dfx: Dfx;

    before(async () => {
        dfx = await deploy({apps: [App.ICRC1Oracle]});
    });

    it("Set operator", async function () {
        const identity = getIdentity("87654321876543218765432187654311");
        const notAdmin = getIdentity("87654321876543218765432187654377");
        let dffActor = await getActor(dfx.icrc1_oracle.id, notAdmin, idlFactory);
        try {
            await dffActor.set_operator(notAdmin.getPrincipal())
            fail("Should throw an error")
        } catch (e) {
            expect(e.message).contains("Unauthorized")
        }
        DFX.ADD_CONTROLLER(identity.getPrincipal().toText(), "icrc1_oracle")
        dfx.icrc1_oracle.actor = await getActor(dfx.icrc1_oracle.id, identity, idlFactory);
        await dfx.icrc1_oracle.actor.set_operator(identity.getPrincipal())
    });

    it("Store/retrieve canister id", async function () {
        let firstCanister: ICRC1 = {
            logo: ["logo"],
            name: "name",
            ledger: "ryjl3-tyaaa-aaaaa-aaaba-cai",
            index: ["irshc-3aaaa-aaaam-absla-cai"],
            symbol: "symbol",
            category: {Spam: null},
            fee: BigInt(1),
            decimals: 1,
            root_canister_id: [],
            date_added: BigInt(Date.now())
        }
        await dfx.icrc1_oracle.actor.store_icrc1_canister(firstCanister);
        let allCanisters = await dfx.icrc1_oracle.actor.get_all_icrc1_canisters() as Array<ICRC1>;
        expect(allCanisters.length).eq(1);
        expect(allCanisters[0].ledger).eq("ryjl3-tyaaa-aaaaa-aaaba-cai");
        expect(allCanisters[0].name).eq("name");
        expect(allCanisters[0].symbol).eq("symbol");
        expect(allCanisters[0].index).deep.eq(["irshc-3aaaa-aaaam-absla-cai"]);
        expect(allCanisters[0].logo).deep.eq(["logo"]);
        expect(allCanisters[0].category).deep.eq({Community: null});

        const secondCanister: ICRC1 = {
            logo: ["logo2"],
            name: "name2",
            ledger: "irshc-3aaaa-aaaam-absla-cai",
            index: ["ryjl3-tyaaa-aaaaa-aaaba-cai"],
            symbol: "symbol2",
            category: {Spam: null},
            fee: BigInt(1),
            decimals: 1,
            root_canister_id: [],
            date_added: BigInt(Date.now())
        }
        const third: ICRC1 = {
            logo: ["logo3"],
            name: "name3",
            ledger: "c543j-2qaaa-aaaal-ac4dq-cai",
            index: ["ryjl3-tyaaa-aaaaa-aaaba-cai"],
            symbol: "symbol3",
            category: {Spam: null},
            fee: BigInt(1),
            decimals: 1,
            root_canister_id: [],
            date_added: BigInt(Date.now())
        }
        firstCanister = allCanisters[0]
        firstCanister.category = {Known: null}
        await dfx.icrc1_oracle.actor.replace_icrc1_canisters([firstCanister, secondCanister, third]);
        allCanisters = await dfx.icrc1_oracle.actor.get_all_icrc1_canisters() as Array<ICRC1>;
        expect(allCanisters.length).eq(3);
        expect(allCanisters.find((k) => k.ledger === firstCanister.ledger).category).deep.eq({Known: null});
    })

    it("Count/getPaginated ICRC1", async function () {
        let canisters = await dfx.icrc1_oracle.actor.count_icrc1_canisters() as number;
        expect(canisters).eq(3n);
        let b = await dfx.icrc1_oracle.actor.get_icrc1_paginated(0, 2) as Array<ICRC1>;
        expect(b.length).eq(2);
        const offset = 2;
        let amountOfRequests = Math.ceil(Number(canisters) / offset);
        expect(amountOfRequests).eq(2);
        const all = await Promise.all(Array.from({length: amountOfRequests}, (_, i) =>
            dfx.icrc1_oracle.actor.get_icrc1_paginated(i * offset, offset)
        )).then((res) => res.flat());
        expect(all.length).eq(3);
    });

    it("Remove ICRC1", async function () {
        let allCanisters = await dfx.icrc1_oracle.actor.get_all_icrc1_canisters() as Array<ICRC1>;
        expect(allCanisters.length).eq(3);
        await dfx.icrc1_oracle.actor.remove_icrc1_canister(allCanisters[0].ledger);
        allCanisters = await dfx.icrc1_oracle.actor.get_all_icrc1_canisters() as Array<ICRC1>;
        expect(allCanisters.length).eq(2);
    })

    it("Replace neurons", async function () {
        let neurons: Array<NeuronData> = [
            {
                name: "name",
                date_added: BigInt(Date.now()),
                ledger: "ledger",
                neuron_id: "neuron_id"
            },
            {
                name: "name2",
                date_added: BigInt(Date.now()),
                ledger: "ledger2",
                neuron_id: "neuron_id2"
            }
        ]
        await dfx.icrc1_oracle.actor.replace_all_neurons(neurons);
        let allNeurons = await dfx.icrc1_oracle.actor.get_all_neurons() as Array<NeuronData>;
        expect(allNeurons.length).eq(2);
    })

    it("Store/get discovery apps (client)", async function () {
        const app1: DiscoveryApp = {
            id: 1,
            derivation_origin: [],
            hostname: "app1.example.com",
            url: ["https://app1.example.com"],
            name: ["App One"],
            image: ["https://app1.example.com/image.png"],
            desc: ["First test app"],
            is_global: false,
            is_anonymous: false,
            unique_users: 0n,
            status: { New: null },
        };
        const app2: DiscoveryApp = {
            id: 2,
            derivation_origin: ["https://origin.example.com"],
            hostname: "app2.example.com",
            url: [],
            name: ["App Two"],
            image: [],
            desc: [],
            is_global: false,
            is_anonymous: false,
            unique_users: 0n,
            status: { New: null },
        };
        await dfx.icrc1_oracle.actor.replace_all_discovery_app([app1, app2]);

        const visit1: DiscoveryVisitRequest = {
            derivation_origin: [],
            hostname: "app1.example.com",
            login: { Global: null },
        };
        await dfx.icrc1_oracle.actor.store_discovery_app(visit1);

        let page = await dfx.icrc1_oracle.actor.get_discovery_app_paginated(0n, 10n) as Array<DiscoveryApp>;
        const found1 = page.find((a) => a.id === 1);
        expect(found1).to.exist;
        expect(found1.unique_users).eq(1n);
        expect(found1.is_global).eq(true);
        expect(found1.is_anonymous).eq(false);

        await dfx.icrc1_oracle.actor.store_discovery_app(visit1);
        page = await dfx.icrc1_oracle.actor.get_discovery_app_paginated(0n, 10n) as Array<DiscoveryApp>;
        expect(page.find((a) => a.id === 1).unique_users).eq(1n);

        const visit1Anon: DiscoveryVisitRequest = {
            derivation_origin: [],
            hostname: "app1.example.com",
            login: { Anonymous: null },
        };
        await dfx.icrc1_oracle.actor.store_discovery_app(visit1Anon);
        page = await dfx.icrc1_oracle.actor.get_discovery_app_paginated(0n, 10n) as Array<DiscoveryApp>;
        const updated1 = page.find((a) => a.id === 1);
        expect(updated1.unique_users).eq(1n);
        expect(updated1.is_anonymous).eq(true);

        const visitUnknown: DiscoveryVisitRequest = {
            derivation_origin: [],
            hostname: "unknown.example.com",
            login: { Global: null },
        };
        await dfx.icrc1_oracle.actor.store_discovery_app(visitUnknown);
        page = await dfx.icrc1_oracle.actor.get_discovery_app_paginated(0n, 10n) as Array<DiscoveryApp>;
        expect(page.length).eq(2);

        const firstPage = await dfx.icrc1_oracle.actor.get_discovery_app_paginated(0n, 1n) as Array<DiscoveryApp>;
        expect(firstPage.length).eq(1);
        const secondPage = await dfx.icrc1_oracle.actor.get_discovery_app_paginated(1n, 1n) as Array<DiscoveryApp>;
        expect(secondPage.length).eq(1);
        expect(firstPage[0].id).not.eq(secondPage[0].id);
    })

    it("Replace all discovery apps (admin)", async function () {
        const apps: DiscoveryApp[] = [
            {
                id: 10,
                derivation_origin: [],
                hostname: "admin-app.example.com",
                url: ["https://admin-app.example.com"],
                name: ["Admin App"],
                image: [],
                desc: ["Admin-replaced app"],
                is_global: true,
                is_anonymous: false,
                unique_users: 999n,
                status: { Verified: null },
            },
        ];

        await dfx.icrc1_oracle.actor.clear_discovery_apps();
        await dfx.icrc1_oracle.actor.replace_all_discovery_app(apps);
        const page = await dfx.icrc1_oracle.actor.get_discovery_app_paginated(0n, 10n) as Array<DiscoveryApp>;
        expect(page.length).eq(1);
        expect(page[0].id).eq(10);
        expect(page[0].hostname).eq("admin-app.example.com");
        expect(page[0].unique_users).eq(999n);
    })

    it("Replace all discovery apps - unauthorised", async function () {
        const notAdmin = getIdentity("87654321876543218765432187654377");
        const actor = await getActor(dfx.icrc1_oracle.id, notAdmin, idlFactory);
        try {
            await actor.replace_all_discovery_app([]);
            fail("Should throw an error");
        } catch (e) {
            expect(e.message).contains("Unauthorised");
        }
    })

    it("is_unique query", async function () {
        const app: DiscoveryApp = {
            id: 20,
            derivation_origin: [],
            hostname: "unique-test.example.com",
            url: [], name: ["Unique Test"], image: [], desc: [],
            is_global: false, is_anonymous: false, unique_users: 0n,
            status: { New: null },
        };
        await dfx.icrc1_oracle.actor.replace_all_discovery_app([app]);

        const visit: DiscoveryVisitRequest = {
            derivation_origin: [],
            hostname: "unique-test.example.com",
            login: { Global: null },
        };

        let needsUpdate = await dfx.icrc1_oracle.actor.is_unique(visit) as boolean;
        expect(needsUpdate).eq(true);

        await dfx.icrc1_oracle.actor.store_discovery_app(visit);
        needsUpdate = await dfx.icrc1_oracle.actor.is_unique(visit) as boolean;
        expect(needsUpdate).eq(false);

        const visitAnon: DiscoveryVisitRequest = {
            derivation_origin: [],
            hostname: "unique-test.example.com",
            login: { Anonymous: null },
        };
        needsUpdate = await dfx.icrc1_oracle.actor.is_unique(visitAnon) as boolean;
        expect(needsUpdate).eq(true);

        await dfx.icrc1_oracle.actor.store_discovery_app(visitAnon);
        needsUpdate = await dfx.icrc1_oracle.actor.is_unique(visitAnon) as boolean;
        expect(needsUpdate).eq(false);

        const visitUnknown: DiscoveryVisitRequest = {
            derivation_origin: [],
            hostname: "no-such-app.example.com",
            login: { Global: null },
        };
        needsUpdate = await dfx.icrc1_oracle.actor.is_unique(visitUnknown) as boolean;
        expect(needsUpdate).eq(false);
    })

    it("store_discovery_app - visit to unknown hostname is a no-op", async function () {
        const visit: DiscoveryVisitRequest = {
            derivation_origin: [],
            hostname: "does-not-exist.example.com",
            login: { Global: null },
        };
        await dfx.icrc1_oracle.actor.store_discovery_app(visit);
    })

})
