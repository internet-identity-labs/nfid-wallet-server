import lombok.SneakyThrows;
import lombok.extern.slf4j.Slf4j;
import org.ic4j.agent.*;
import org.ic4j.agent.http.ReplicaJavaHttpTransport;
import org.ic4j.agent.identity.BasicIdentity;
import org.ic4j.candid.parser.IDLArgs;
import org.ic4j.candid.parser.IDLValue;
import org.ic4j.candid.pojo.PojoDeserializer;
import org.ic4j.candid.pojo.PojoSerializer;
import org.ic4j.candid.types.Type;
import org.ic4j.internetidentity.*;
import org.ic4j.types.Principal;
import org.testng.annotations.BeforeClass;
import org.testng.annotations.Ignore;
import org.testng.annotations.Test;
import org.testng.internal.collections.Pair;
import records.HTTPAccessPointRequest;
import records.HTTPAccountRequest;
import records.HttpResponse;
import records.Protection;

import java.nio.file.Paths;
import java.util.Arrays;
import java.util.Optional;

import records.DeviceData;

import static org.testng.AssertJUnit.assertEquals;
import static org.testng.AssertJUnit.assertTrue;

@Slf4j
@Ignore
public class InternetIdentityConnectionITest extends BaseDFXITest {
    private BasicIdentity identity;
    private Agent agent;
    private InternetIdentityProxy iiProxy;
    private Principal im;
    private Principal ii;

    @SneakyThrows
    @BeforeClass
    public void init() {
        int i = 0;
        String configureResponse;
        do {
            call("common/dfx_stop");
            callDfxCommand("rm -rf .dfx");
            call("common/use_admin_persona");
            ROOT_IDENTITY = call("common/get_principal").trim();

            call("common/init_dfx_project_full");
            call("common/deploy_ii");
            call("common/deploy_im");

            String imCanisterId = call("common/get_canister_id", "identity_manager").trim();
            String iiCanisterId = call("common/get_canister_id", "internet_identity_test").trim();
            configureResponse = callDfxCommand(String.format("dfx canister call %s configure '(record {ii_canister_id = opt principal \"%s\"; })'", "identity_manager", iiCanisterId));

            if (!configureResponse.isEmpty()) {
                String icLocation = "http://localhost:8000";
                ReplicaTransport transport = ReplicaJavaHttpTransport.create(icLocation);
                identity = BasicIdentity.fromPEMFile(Paths.get(this.getClass().getClassLoader().getResource("identity/" + "identity.pem").getPath()));
                agent = new AgentBuilder().transport(transport).identity(identity).build();
                im = Principal.fromString(imCanisterId);
                ii = Principal.fromString(iiCanisterId);

                iiProxy = ProxyBuilder.create(agent, ii)
                        .getProxy(InternetIdentityProxy.class);
            }

            if (++i >= 5) {
                call("common/dfx_stop");
                log.error("Stopping ApplicationITest");
                System.exit(1);
            }
        } while (configureResponse.isEmpty());
        log.info("II tests are configurated.");
    }


    @Test(priority = 10)
    public void createAccountWhenError() {
        var accountRequest = new HTTPAccountRequest(10001l);

        try {
            callUpdateHttp("create_account", im, IDLValue.create(accountRequest, new PojoSerializer()));
            assertTrue(false);
        } catch (Exception e) {
            assertTrue(e.getMessage().contains("could not be authenticated"));
        }
    }

    @Test(priority = 20)
    @SneakyThrows
    public void createAccountWhenOk() {
        register();
        var accountRequest = new HTTPAccountRequest(10000L);

        var accountResponse = callUpdateHttp("create_account", im, IDLValue.create(accountRequest, new PojoSerializer()));
        assertEquals(200, accountResponse.statusCode.intValue());
        assertTrue(accountResponse.error.isEmpty());
        assertTrue(accountResponse.data.isPresent());
    }

    @Test(priority = 30)
    public void createAccessPointWhenError() {
        var pbk = identity.derEncodedPublickey.clone();
        pbk[0] = 0;
        var accessPointRequest = new HTTPAccessPointRequest(pbk, "icon", "device", "browser");

        try {
            callUpdateHttp("create_access_point", im, IDLValue.create(accessPointRequest, new PojoSerializer()));
            assertTrue(false);
        } catch (Exception e) {
            assertTrue(e.getMessage().contains("could not be authenticated"));
        }
    }

    @Test(priority = 40)
    @SneakyThrows
    public void createAccessPointWhenOk() {
        var accessPointRequest = new HTTPAccessPointRequest(identity.derEncodedPublickey, "icon", "device", "browser");
        var response = callUpdateHttp("create_access_point", im, IDLValue.create(accessPointRequest, new PojoSerializer()));

        assertEquals(200, response.statusCode.intValue());
        assertTrue(response.error.isEmpty());
        assertTrue(response.data.isPresent());
    }

    @Test(priority = 70)
    @SneakyThrows
    public void removeAccountExpectOk() {
        var response = (HttpResponse<Boolean>) callUpdateHttp("remove_account", im);

        assertEquals(200, response.statusCode.intValue());
        assertTrue(response.error.isEmpty());
        assertTrue(response.data.isPresent());
    }

    @SneakyThrows
    private void register() {
        var challengeResponse = iiProxy.createChallenge().get();

        var challengeResult = new ChallengeResult();
        challengeResult.challengeKey = challengeResponse.challengeKey;
        challengeResult.chars = "a";

        var deviceData = new DeviceData();
        deviceData.pubkey = identity.derEncodedPublickey;
        deviceData.alias = "Device1";
        deviceData.purpose = Purpose.authentication;
        deviceData.keyType = KeyType.platform;
        deviceData.protection = Protection.unprotected;
        deviceData.credentialId = Optional.empty();

        callUpdateHttp("register", ii, IDLValue.create(deviceData, new PojoSerializer()), IDLValue.create(challengeResult, new PojoSerializer()));
    }

    @SneakyThrows
    private HttpResponse callUpdateHttp(String methodName, Principal canister, IDLValue... idls) {
        var response = UpdateBuilder
                .create(agent, canister, methodName)
                .arg(IDLArgs.create(Arrays.asList(idls)).toBytes())
                .callAndWait(Waiter.create(120, 2));
        return IDLArgs.fromBytes(response.get())
                .getArgs()
                .get(0)
                .getValue(PojoDeserializer.create(), HttpResponse.class);
    }

}
