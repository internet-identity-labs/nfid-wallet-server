import {Dfx} from "./type/dfx";
import {deploy, createIdentityManagerAccount, getTypedActor, getIdentity} from "./util/deployment.util";
import {App} from "./constanst/app.enum";
import {expect} from "chai";
import {AddressBookUserAddress, AddressBookConf, _SERVICE as UserRegistry} from "./idl/user_registry";
import {DFX} from "./constanst/dfx.const";
import {AccessPointRequest} from "./idl/identity_manager";
import {idlFactory as userRegistryIdl} from "./idl/user_registry_idl";

describe("User Registry", () => {
    var dfx: Dfx;

    before(async () => {
        dfx = await deploy({apps: [App.IdentityManager, App.UserRegistry]});
    });

    describe("ICRC1 canister Storage", () => {
        let canister_id = "id1";
        let one_more_canister_id = "id2";

        it("Store/retrieve canister id", async function () {
            await dfx.user_registry.actor.store_icrc1_canister(canister_id, { 'Active': null }, []);
            await dfx.user_registry.actor.store_icrc1_canister(one_more_canister_id, { 'Inactive': null }, [1]);
            let canisters = await dfx.user_registry.actor.get_canisters_by_root(dfx.user.identity.getPrincipal().toText());
            expect(canisters.length).eq(2);
            expect(canisters.find((c) => c.ledger === canister_id)?.state).deep.eq({ 'Active': null });
            expect(canisters.find((c) => c.ledger === one_more_canister_id)?.state).deep.eq({ 'Inactive': null });
            await dfx.user_registry.actor.store_icrc1_canister(canister_id, { 'Inactive': null }, []);
            canisters = await dfx.user_registry.actor.get_canisters_by_root(dfx.user.identity.getPrincipal().toText());
            expect(canisters.find((c) => c.ledger === canister_id).state).deep.eq({ 'Inactive': null });
            expect(canisters.find((c) => c.ledger === canister_id).network).eq(0);
            expect(canisters.find((c) => c.ledger === one_more_canister_id).network).eq(1);
        })

        it("Remove canister", async function () {
            await dfx.user_registry.actor.remove_icrc1_canister(canister_id);
            let canisters = await dfx.user_registry.actor.get_canisters_by_root(dfx.user.identity.getPrincipal().toText());
            expect(canisters.length).eq(1);
            expect(canisters[0].ledger).eq(one_more_canister_id);
        })
    });

    describe("Address Book", () => {

        before(async () => {
            DFX.DEPLOY_WITH_ARGUMENT("user_registry", `(record { im_canister = opt "${dfx.im.id}" })`);
            const actor = await createIdentityManagerAccount(dfx)
            const addressBookAccessPoint: AccessPointRequest = {
                icon: "Passkey",
                device: "Passkey",
                pub_key: dfx.user.principal,
                browser: "Passkey",
                device_type: {
                    Passkey: null,
                },
                credential_id: [],
            };
            await actor.create_access_point(addressBookAccessPoint);
        });

        beforeEach(async () => {
            await dfx.user_registry.actor.address_book_delete_all();

            const config: AddressBookConf = {
                max_user_addresses: 2,
                max_name_length: 50
            };
            await dfx.user_registry.actor.address_book_set_config(config);
        });

        it("should save a new address successfully", async function () {
            // Given
            const address: AddressBookUserAddress = {
                id: "addr1",
                name: "My Bitcoin Wallet",
                addresses: [
                    { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" }
                ]
            };

            // When
            const result = await dfx.user_registry.actor.address_book_save(address);

            // Then
            expect(result).to.have.property('Ok');
            const addresses = (result as { Ok: Array<typeof address> }).Ok;
            expect(addresses.length).eq(1);
            expect(addresses[0]).to.deep.equal(address);
        });

        it("should replace an existing address when saving with the same id", async function () {
            // Given
            const originalAddress: AddressBookUserAddress = {
                id: "addr1",
                name: "Original Name",
                addresses: [
                    { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" }
                ]
            };
            await dfx.user_registry.actor.address_book_save(originalAddress);

            // When
            const updatedAddress: AddressBookUserAddress = {
                id: "addr1",
                name: "Updated Name",
                addresses: [
                    { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" },
                    { address_type: { 'ETH': null }, value: "0x123abc" }
                ]
            };
            const result = await dfx.user_registry.actor.address_book_save(updatedAddress);

            // Then
            expect(result).to.have.property('Ok');
            const addresses = (result as { Ok: Array<typeof updatedAddress> }).Ok;
            expect(addresses.length).eq(1);
            expect(addresses[0]).to.deep.equal(updatedAddress);
        });

        it("should save a second address with different id", async function () {
            // Given
            const address1: AddressBookUserAddress = {
                id: "addr1",
                name: "First Wallet",
                addresses: [
                    { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" }
                ]
            };
            await dfx.user_registry.actor.address_book_save(address1);

            // When
            const address2: AddressBookUserAddress = {
                id: "addr2",
                name: "Second Wallet",
                addresses: [
                    { address_type: { 'ETH': null }, value: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb" }
                ]
            };
            const result = await dfx.user_registry.actor.address_book_save(address2);

            // Then
            expect(result).to.have.property('Ok');
            const addresses = (result as { Ok: Array<typeof address2> }).Ok;
            expect(addresses.length).eq(2);
        });

        it("should return an error when saving with duplicate addresses", async function () {
            // Given
            const duplicateAddress: AddressBookUserAddress = {
                id: "addr1",
                name: "Duplicate Address Test",
                addresses: [
                    { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" },
                    { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" }
                ]
            };

            // When
            const result = await dfx.user_registry.actor.address_book_save(duplicateAddress);

            // Then
            expect(result).to.have.nested.property('Err.DuplicateAddress');
        });

        it("should return an error when saving two user addresses with the same address value", async function () {
            // Given
            const address1: AddressBookUserAddress = {
                id: "addr1",
                name: "First Bitcoin Wallet",
                addresses: [
                    { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" }
                ]
            };
            await dfx.user_registry.actor.address_book_save(address1);

            // When
            const address2: AddressBookUserAddress = {
                id: "addr2",
                name: "Second Bitcoin Wallet",
                addresses: [
                    { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" }
                ]
            };
            const result = await dfx.user_registry.actor.address_book_save(address2);

            // Then
            expect(result).to.have.nested.property('Err.DuplicateAddress');
            const addressesResult = await dfx.user_registry.actor.address_book_find_all();
            expect(addressesResult).to.have.nested.property('Ok.length', 1);
            expect(addressesResult).to.have.nested.property('Ok[0]').that.deep.equals(address1);
        });

        it("should return an error when saving a new address with a duplicate name", async function () {
            // Given
            const address1: AddressBookUserAddress = {
                id: "addr1",
                name: "My Wallet",
                addresses: [
                    { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" }
                ]
            };
            await dfx.user_registry.actor.address_book_save(address1);

            // When
            const address2: AddressBookUserAddress = {
                id: "addr2",
                name: "My Wallet",
                addresses: [
                    { address_type: { 'ETH': null }, value: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb" }
                ]
            };
            const result = await dfx.user_registry.actor.address_book_save(address2);

            // Then
            expect(result).to.have.nested.property('Err.DuplicateName');
            const addressesResult = await dfx.user_registry.actor.address_book_find_all();
            expect(addressesResult).to.have.nested.property('Ok.length', 1);
            expect(addressesResult).to.have.nested.property('Ok[0]').that.deep.equals(address1);
        });

        it("should return an error when updating an address name to a duplicate", async function () {
            // Given
            const address1: AddressBookUserAddress = {
                id: "addr1",
                name: "First Wallet",
                addresses: [
                    { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" }
                ]
            };
            const address2: AddressBookUserAddress = {
                id: "addr2",
                name: "Second Wallet",
                addresses: [
                    { address_type: { 'ETH': null }, value: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb" }
                ]
            };
            await dfx.user_registry.actor.address_book_save(address1);
            await dfx.user_registry.actor.address_book_save(address2);

            // When
            const updatedAddress2: AddressBookUserAddress = {
                id: "addr2",
                name: "First Wallet",
                addresses: [
                    { address_type: { 'ETH': null }, value: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb" }
                ]
            };
            const result = await dfx.user_registry.actor.address_book_save(updatedAddress2);

            // Then
            expect(result).to.have.nested.property('Err.DuplicateName');
            const addressesResult = await dfx.user_registry.actor.address_book_find_all();
            expect(addressesResult).to.have.nested.property('Ok.length', 2);
            expect(addressesResult).to.have.nested.property('Ok').that.deep.includes(address1);
            expect(addressesResult).to.have.nested.property('Ok').that.deep.includes(address2);
        });

        it("should find all saved addresses", async function () {
            // Given
            const address1: AddressBookUserAddress = {
                id: "addr1",
                name: "Bitcoin Wallet",
                addresses: [
                    { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" }
                ]
            };
            const address2: AddressBookUserAddress = {
                id: "addr2",
                name: "Ethereum Wallet",
                addresses: [
                    { address_type: { 'ETH': null }, value: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb" }
                ]
            };
            await dfx.user_registry.actor.address_book_save(address1);
            await dfx.user_registry.actor.address_book_save(address2);

            // When
            const result = await dfx.user_registry.actor.address_book_find_all();

            // Then
            expect(result).to.have.property('Ok');
            const addresses = (result as { Ok: Array<typeof address1> }).Ok;
            expect(addresses).to.have.deep.members([address1, address2]);
        });

        it("should delete an address successfully when it exists", async function () {
            // Given
            const addressId2 = "addr2";
            const address1: AddressBookUserAddress = {
                id: "addr1",
                name: "First Wallet",
                addresses: [
                    { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" }
                ]
            };
            const address2: AddressBookUserAddress = {
                id: addressId2,
                name: "Second Wallet",
                addresses: [
                    { address_type: { 'ETH': null }, value: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb" }
                ]
            };
            await dfx.user_registry.actor.address_book_save(address1);
            await dfx.user_registry.actor.address_book_save(address2);

            // When
            const result = await dfx.user_registry.actor.address_book_delete(addressId2);

            // Then
            expect(result).to.have.property('Ok');
            const addresses = (result as { Ok: Array<typeof address1> }).Ok;
            expect(addresses.length).eq(1);
            expect(addresses[0]).to.deep.equal(address1);
        });

        it("should return an error when deleting a non-existent address", async function () {
            // Given - empty address book
            const nonExistentId = "nonexistent";

            // When
            const result = await dfx.user_registry.actor.address_book_delete(nonExistentId);

            // Then
            expect(result).to.have.nested.property('Err.AddressNotFound');
        });

        it("should return an error when saving an address with name exceeding max_name_length", async function () {
            // Given
            const longName = "a".repeat(51); // Create a name with 201 characters
            const longNameAddress: AddressBookUserAddress = {
                id: "addr1",
                name: longName,
                addresses: [
                    { address_type: { 'IcpAddress': null }, value: "abc123" }
                ]
            };

            // When
            const result = await dfx.user_registry.actor.address_book_save(longNameAddress);

            // Then
            expect(result).to.have.nested.property('Err.NameTooLong');
        });

        it("should return an error when saving exceeds max_user_addresses limit", async function () {
            // Given
            const address1: AddressBookUserAddress = {
                id: "addr1",
                name: "First Address",
                addresses: [
                    { address_type: { 'BTC': null }, value: "address1" }
                ]
            };
            const address2: AddressBookUserAddress = {
                id: "addr2",
                name: "Second Address",
                addresses: [
                    { address_type: { 'BTC': null }, value: "address2" }
                ]
            };
            await dfx.user_registry.actor.address_book_save(address1);
            await dfx.user_registry.actor.address_book_save(address2);

            // When
            const address3: AddressBookUserAddress = {
                id: "addr3",
                name: "Third Address",
                addresses: [
                    { address_type: { 'BTC': null }, value: "address3" }
                ]
            };
            const result = await dfx.user_registry.actor.address_book_save(address3);

            // Then
            expect(result).to.have.nested.property('Err.MaxAddressesReached');
            const addressesResult = await dfx.user_registry.actor.address_book_find_all();
            expect(addressesResult).to.have.nested.property('Ok.length', 2);
        });

        it("should persist address book data after canister upgrade", async function () {
            // Given
            const addressId1 = "addr1";
            const address1: AddressBookUserAddress = {
                id: addressId1,
                name: "Persist Address 1",
                addresses: [
                    { address_type: { 'IcpPrincipal': null }, value: "principal123" }
                ]
            };
            const address2: AddressBookUserAddress = {
                id: "addr2",
                name: "Persist Address 2",
                addresses: [
                    { address_type: { 'ETH': null }, value: "eth456" }
                ]
            };
            await dfx.user_registry.actor.address_book_save(address1);
            await dfx.user_registry.actor.address_book_save(address2);

            // When
            DFX.UPGRADE_WITH_ARGUMENT('user_registry', `(record { im_canister = opt "${dfx.im.id}" })`);

            // Then
            const addressesAfterResult = await dfx.user_registry.actor.address_book_find_all();
            expect(addressesAfterResult).to.have.property('Ok');
            const addresses = (addressesAfterResult as { Ok: Array<typeof address1> }).Ok;
            expect(addresses).to.have.deep.members([address1, address2]);
        });

        it("should return the default address book configuration", async function () {
            // Given - address book uses default config (im_canister not set, so max_user_addresses is 2)
            const expectedConfig = {
                max_user_addresses: 2,
                max_name_length: 50
            };

            // When
            const config = await dfx.user_registry.actor.address_book_get_config();

            // Then
            expect(config).to.deep.equal(expectedConfig);
        });

        it("should persist address book configuration after canister upgrade", async function () {
            // Given - default configuration (im_canister not set, so max_user_addresses is 2)
            const expectedConfig = {
                max_user_addresses: 2,
                max_name_length: 50
            };

            // When
            DFX.UPGRADE_WITH_ARGUMENT('user_registry', `(record { im_canister = opt "${dfx.im.id}" })`);

            // Then
            const configAfter = await dfx.user_registry.actor.address_book_get_config();
            expect(configAfter).to.deep.equal(expectedConfig);
        });

        it("should return Unauthorized error when non-controller tries to set config", async function () {
            // Given
            const nonExistentIdentity = getIdentity("87654321876543218765432187654313");
            const nonExistentActor = await getTypedActor<UserRegistry>(dfx.user_registry.id, nonExistentIdentity, userRegistryIdl);

            const newConfig: AddressBookConf = {
                max_user_addresses: 100,
                max_name_length: 500
            };

            // When
            const result = await nonExistentActor.address_book_set_config(newConfig);

            // Then
            expect(result).to.have.nested.property('Err.Unauthorized');
            const currentConfig = await dfx.user_registry.actor.address_book_get_config();
            expect(currentConfig).to.not.deep.equal(newConfig);
        });

        it("should allow controller to set config successfully", async function () {
            // Given
            const newConfig: AddressBookConf = {
                max_user_addresses: 75,
                max_name_length: 300
            };

            // When
            const result = await dfx.user_registry.actor.address_book_set_config(newConfig);

            // Then
            expect(result).to.have.property('Ok');
            const currentConfig = await dfx.user_registry.actor.address_book_get_config();
            expect(currentConfig).to.deep.equal(newConfig);
        });

        it("should trap when user does not exist in IdentityManager", async function () {
            // Given
            const nonExistentIdentity = getIdentity("87654321876543218765432187654313");
            const nonExistentActor = await getTypedActor<UserRegistry>(dfx.user_registry.id, nonExistentIdentity, userRegistryIdl);

            const newConfig: AddressBookConf = {
                max_user_addresses: 100,
                max_name_length: 500
            };
            await dfx.user_registry.actor.address_book_set_config(newConfig);

            const address: AddressBookUserAddress = {
                id: "addr1",
                name: "Test Wallet",
                addresses: [
                    { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" }
                ]
            };

            // When
            const savePromise = nonExistentActor.address_book_save(address);
            const findAllPromise = nonExistentActor.address_book_find_all();
            const deletePromise = nonExistentActor.address_book_delete("addr1");
            const deleteAllPromise = nonExistentActor.address_book_delete_all();

            // Then
            try {
                await savePromise;
                expect.fail("Should have trapped");
            } catch (error) {
                expect(error.message).to.include("No root found for this principal");
            }

            try {
                await findAllPromise;
                expect.fail("Should have trapped");
            } catch (error) {
                expect(error.message).to.include("No root found for this principal");
            }

            try {
                await deletePromise;
                expect.fail("Should have trapped");
            } catch (error) {
                expect(error.message).to.include("No root found for this principal");
            }

            try {
                await deleteAllPromise;
                expect.fail("Should have trapped");
            } catch (error) {
                expect(error.message).to.include("No root found for this principal");
            }
        });
    });
});
