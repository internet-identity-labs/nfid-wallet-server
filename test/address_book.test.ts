import {Dfx} from "./type/dfx";
import {deploy, getIdentity, getTypedActor} from "./util/deployment.util";
import {App} from "./constanst/app.enum";
import {expect} from "chai";
import {UserAddress, Conf, _SERVICE as AddressBookService} from "./idl/address_book";
import {DFX} from "./constanst/dfx.const";
import {idlFactory as addressBookIDL} from "./idl/address_book_idl";

describe("Address Book", () => {
    var dfx: Dfx;

    before(async () => {
        dfx = await deploy({apps: [App.AddressBook]});
    });

    beforeEach(async () => {
        await dfx.address_book.actor.delete_all();
        await dfx.address_book.actor.set_config({
            max_user_addresses: 2,
            max_name_length: 30
        });
    });

    it("should save a new address successfully", async function () {
        // Given
        const address: UserAddress = {
            id: "addr1",
            name: "My Bitcoin Wallet",
            addresses: [
                { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" }
            ]
        };

        // When
        const result = await dfx.address_book.actor.save(address);

        // Then
        expect(result).to.have.property('Ok');
        const addresses = (result as { Ok: Array<typeof address> }).Ok;
        expect(addresses.length).eq(1);
        expect(addresses[0]).to.deep.equal(address);
    });

    it("should replace an existing address when saving with the same id", async function () {
        // Given
        const originalAddress: UserAddress = {
            id: "addr1",
            name: "Original Name",
            addresses: [
                { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" }
            ]
        };
        await dfx.address_book.actor.save(originalAddress);

        // When
        const updatedAddress: UserAddress = {
            id: "addr1",
            name: "Updated Name",
            addresses: [
                { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" },
                { address_type: { 'ETH': null }, value: "0x123abc" }
            ]
        };
        const result = await dfx.address_book.actor.save(updatedAddress);

        // Then
        expect(result).to.have.property('Ok');
        const addresses = (result as { Ok: Array<typeof updatedAddress> }).Ok;
        expect(addresses.length).eq(1);
        expect(addresses[0]).to.deep.equal(updatedAddress);
    });

    it("should save a second address with different id", async function () {
        // Given
        const address1: UserAddress = {
            id: "addr1",
            name: "First Wallet",
            addresses: [
                { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" }
            ]
        };
        await dfx.address_book.actor.save(address1);

        // When
        const address2: UserAddress = {
            id: "addr2",
            name: "Second Wallet",
            addresses: [
                { address_type: { 'ETH': null }, value: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb" }
            ]
        };
        const result = await dfx.address_book.actor.save(address2);

        // Then
        expect(result).to.have.property('Ok');
        const addresses = (result as { Ok: Array<typeof address2> }).Ok;
        expect(addresses.length).eq(2);
    });

    it("should return an error when saving with duplicate addresses", async function () {
        // Given
        const duplicateAddress: UserAddress = {
            id: "addr1",
            name: "Duplicate Address Test",
            addresses: [
                { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" },
                { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" }
            ]
        };

        // When
        const result = await dfx.address_book.actor.save(duplicateAddress);

        // Then
        expect(result).to.have.nested.property('Err.DuplicateAddress');
    });

    it("should return an error when saving two user addresses with the same address value", async function () {
        // Given
        const address1: UserAddress = {
            id: "addr1",
            name: "First Bitcoin Wallet",
            addresses: [
                { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" }
            ]
        };
        await dfx.address_book.actor.save(address1);

        // When
        const address2: UserAddress = {
            id: "addr2",
            name: "Second Bitcoin Wallet",
            addresses: [
                { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" }
            ]
        };
        const result = await dfx.address_book.actor.save(address2);

        // Then
        expect(result).to.have.nested.property('Err.DuplicateAddress');
        const addressesResult = await dfx.address_book.actor.find_all();
        expect(addressesResult).to.have.nested.property('Ok.length', 1);
        expect(addressesResult).to.have.nested.property('Ok[0]').that.deep.equals(address1);
    });

    it("should return an error when saving a new address with a duplicate name", async function () {
        // Given
        const address1: UserAddress = {
            id: "addr1",
            name: "My Wallet",
            addresses: [
                { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" }
            ]
        };
        await dfx.address_book.actor.save(address1);

        // When
        const address2: UserAddress = {
            id: "addr2",
            name: "My Wallet",
            addresses: [
                { address_type: { 'ETH': null }, value: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb" }
            ]
        };
        const result = await dfx.address_book.actor.save(address2);

        // Then
        expect(result).to.have.nested.property('Err.DuplicateName');
        const addressesResult = await dfx.address_book.actor.find_all();
        expect(addressesResult).to.have.nested.property('Ok.length', 1);
        expect(addressesResult).to.have.nested.property('Ok[0]').that.deep.equals(address1);
    });

    it("should return an error when updating an address name to a duplicate", async function () {
        // Given
        const address1: UserAddress = {
            id: "addr1",
            name: "First Wallet",
            addresses: [
                { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" }
            ]
        };
        const address2: UserAddress = {
            id: "addr2",
            name: "Second Wallet",
            addresses: [
                { address_type: { 'ETH': null }, value: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb" }
            ]
        };
        await dfx.address_book.actor.save(address1);
        await dfx.address_book.actor.save(address2);

        // When
        const updatedAddress2: UserAddress = {
            id: "addr2",
            name: "First Wallet",
            addresses: [
                { address_type: { 'ETH': null }, value: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb" }
            ]
        };
        const result = await dfx.address_book.actor.save(updatedAddress2);

        // Then
        expect(result).to.have.nested.property('Err.DuplicateName');
        const addressesResult = await dfx.address_book.actor.find_all();
        expect(addressesResult).to.have.nested.property('Ok.length', 2);
        expect(addressesResult).to.have.nested.property('Ok').that.deep.includes(address1);
        expect(addressesResult).to.have.nested.property('Ok').that.deep.includes(address2);
    });

    it("should find all saved addresses", async function () {
        // Given
        const address1: UserAddress = {
            id: "addr1",
            name: "Bitcoin Wallet",
            addresses: [
                { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" }
            ]
        };
        const address2: UserAddress = {
            id: "addr2",
            name: "Ethereum Wallet",
            addresses: [
                { address_type: { 'ETH': null }, value: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb" }
            ]
        };
        await dfx.address_book.actor.save(address1);
        await dfx.address_book.actor.save(address2);

        // When
        const result = await dfx.address_book.actor.find_all();

        // Then
        expect(result).to.have.property('Ok');
        const addresses = (result as { Ok: Array<typeof address1> }).Ok;
        expect(addresses).to.have.deep.members([address1, address2]);
    });

    it("should delete an address successfully when it exists", async function () {
        // Given
        const addressId2 = "addr2";
        const address1: UserAddress = {
            id: "addr1",
            name: "First Wallet",
            addresses: [
                { address_type: { 'BTC': null }, value: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" }
            ]
        };
        const address2: UserAddress = {
            id: addressId2,
            name: "Second Wallet",
            addresses: [
                { address_type: { 'ETH': null }, value: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb" }
            ]
        };
        await dfx.address_book.actor.save(address1);
        await dfx.address_book.actor.save(address2);

        // When
        const result = await dfx.address_book.actor.delete(addressId2);

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
        const result = await dfx.address_book.actor.delete(nonExistentId);

        // Then
        expect(result).to.have.nested.property('Err.AddressNotFound');
    });

    it("should return an error when saving an address with name exceeding max_name_length", async function () {
        // Given
        const longNameAddress: UserAddress = {
            id: "addr1",
            name: "This name is definitely longer than thirty characters",
            addresses: [
                { address_type: { 'IcpAddress': null }, value: "abc123" }
            ]
        };

        // When
        const result = await dfx.address_book.actor.save(longNameAddress);

        // Then
        expect(result).to.have.nested.property('Err.NameTooLong');
    });

    it("should return an error when saving exceeds max_user_addresses limit", async function () {
        // Given
        const address1: UserAddress = {
            id: "addr1",
            name: "First Address",
            addresses: [
                { address_type: { 'BTC': null }, value: "address1" }
            ]
        };
        const address2: UserAddress = {
            id: "addr2",
            name: "Second Address",
            addresses: [
                { address_type: { 'BTC': null }, value: "address2" }
            ]
        };
        await dfx.address_book.actor.save(address1);
        await dfx.address_book.actor.save(address2);

        // When
        const address3: UserAddress = {
            id: "addr3",
            name: "Third Address",
            addresses: [
                { address_type: { 'BTC': null }, value: "address3" }
            ]
        };
        const result = await dfx.address_book.actor.save(address3);

        // Then
        expect(result).to.have.nested.property('Err.MaxAddressesReached');
        const addressesResult = await dfx.address_book.actor.find_all();
        expect(addressesResult).to.have.nested.property('Ok.length', 2);
    });

    it("should persist data after canister upgrade", async function () {
        // Given
        const addressId1 = "addr1";
        const address1: UserAddress = {
            id: addressId1,
            name: "Persist Address 1",
            addresses: [
                { address_type: { 'IcpPrincipal': null }, value: "principal123" }
            ]
        };
        const address2: UserAddress = {
            id: "addr2",
            name: "Persist Address 2",
            addresses: [
                { address_type: { 'ETH': null }, value: "eth456" }
            ]
        };
        await dfx.address_book.actor.save(address1);
        await dfx.address_book.actor.save(address2);

        // When
        DFX.UPGRADE_FORCE('address_book');

        // Then
        const addressesAfterResult = await dfx.address_book.actor.find_all();
        expect(addressesAfterResult).to.have.property('Ok');
        const addresses = (addressesAfterResult as { Ok: Array<typeof address1> }).Ok;
        expect(addresses).to.have.deep.members([address1, address2]);
    });

    it("should return the current configuration", async function () {
        // Given - address book is deployed with specific config
        const expectedConfig = {
            max_user_addresses: 2,
            max_name_length: 30
        };

        // When
        const config = await dfx.address_book.actor.get_config() as Conf;

        // Then
        expect(config).to.deep.include(expectedConfig);
    });

    it("should persist configuration after canister upgrade", async function () {
        // Given
        const config: Conf = {
            max_user_addresses: 4,
            max_name_length: 4
        };
        const result = await dfx.address_book.actor.set_config(config);
        expect(result).to.have.property('Ok');

        // When
        DFX.UPGRADE_FORCE('address_book');

        // Then
        const configAfter = await dfx.address_book.actor.get_config() as Conf;
        expect(configAfter).to.deep.include(config);
    });

    it("should update configuration using set_config", async function () {
        // Given
        // User is already a controller from the before hook

        // When
        const config: Conf = {
            max_user_addresses: 5,
            max_name_length: 100
        };
        const result = await dfx.address_book.actor.set_config(config);

        // Then
        expect(result).to.have.property('Ok');
        const updatedConfig = await dfx.address_book.actor.get_config() as Conf;
        expect(updatedConfig).to.deep.equal(config);
    });

    it("should allow controller to update configuration", async function () {
        // Given
        // User is already a controller from the before hook

        // When
        const config: Conf = {
            max_user_addresses: 10,
            max_name_length: 50
        };
        const result = await dfx.address_book.actor.set_config(config);

        // Then
        expect(result).to.have.property('Ok');
        const updatedConfig = await dfx.address_book.actor.get_config() as Conf;
        expect(updatedConfig).to.deep.equal(config);
    });

    it("should fail to update configuration when caller is not a controller", async function () {
        // Given
        const unauthorizedIdentity = getIdentity("12345678123456781234567812345678");
        const unauthorizedActor = await getTypedActor<AddressBookService>(
            dfx.address_book.id,
            unauthorizedIdentity,
            addressBookIDL
        );

        // When
        const config: Conf = {
            max_user_addresses: 100,
            max_name_length: 500
        };
        const result = await unauthorizedActor.set_config(config);

        // Then
        expect(result).to.have.nested.property('Err.Unauthorized');

        // Verify config was not changed
        const currentConfig = await dfx.address_book.actor.get_config() as Conf;
        expect(currentConfig).to.not.deep.equal(config);
    });

});
