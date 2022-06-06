import static org.testng.AssertJUnit.assertEquals;
import static org.testng.AssertJUnit.assertTrue;

import java.nio.file.Paths;
import java.util.Arrays;

import org.ic4j.agent.Agent;
import org.ic4j.agent.AgentBuilder;
import org.ic4j.agent.ProxyBuilder;
import org.ic4j.agent.ReplicaTransport;
import org.ic4j.agent.UpdateBuilder;
import org.ic4j.agent.Waiter;
import org.ic4j.agent.http.ReplicaJavaHttpTransport;
import org.ic4j.agent.identity.BasicIdentity;
import org.ic4j.candid.parser.IDLArgs;
import org.ic4j.candid.parser.IDLValue;
import org.ic4j.candid.pojo.PojoDeserializer;
import org.ic4j.candid.pojo.PojoSerializer;
import org.ic4j.candid.types.Type;
import org.ic4j.internetidentity.ChallengeResult;
import org.ic4j.internetidentity.DeviceData;
import org.ic4j.internetidentity.InternetIdentityProxy;
import org.ic4j.internetidentity.KeyType;
import org.ic4j.internetidentity.Purpose;
import org.ic4j.types.Principal;
import org.testng.annotations.BeforeClass;
import org.testng.annotations.Test;

import lombok.SneakyThrows;
import lombok.extern.slf4j.Slf4j;
import records.HTTPAccessPointRequest;
import records.HTTPAccountRequest;
import records.HttpResponse;

@Slf4j
public class InternetIdentityConnectionITest extends BaseDFXITest {
  private BasicIdentity identity;
  private Agent agent;
  private InternetIdentityProxy iiProxy;
  private Principal im;

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

          if(!configureResponse.isEmpty()) {
            String icLocation = "http://localhost:8000";
            ReplicaTransport transport = ReplicaJavaHttpTransport.create(icLocation);
            identity = BasicIdentity.fromPEMFile(Paths.get(this.getClass().getClassLoader().getResource("identity/" + "identity.pem").getPath()));
            agent = new AgentBuilder().transport(transport).identity(identity).build();
            im = Principal.fromString(imCanisterId);

            iiProxy = ProxyBuilder.create(agent, Principal.fromString(iiCanisterId))
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
  public void createAccount_whenError() {
    var accountRequest = new HTTPAccountRequest(10001l);
    
    try {
      callUpdateHttp("create_account", IDLValue.create(accountRequest, new PojoSerializer()));
      assertTrue(false);
    } catch (Exception e) {
      assertTrue(e.getMessage().contains("could not be authenticated"));
    }
  }

  @Test(priority = 20)
  @SneakyThrows
  public void createAccount_whenOk() {
    var anchor = register();
    var accountRequest = new HTTPAccountRequest(anchor);

    var accountResponse = callUpdateHttp("create_account", IDLValue.create(accountRequest, new PojoSerializer()));
    assertEquals(200, accountResponse.statusCode.intValue());
    assertTrue(accountResponse.error.isEmpty());
    assertTrue(accountResponse.data.isPresent());
  }

  @Test(priority = 30)
  public void createAccessPoint_whenError() {
    var pbk = identity.derEncodedPublickey.clone();
    pbk[0] = 0;
    var accessPointRequest = new HTTPAccessPointRequest(pbk, "icon", "device", "browser");

    try {
      callUpdateHttp("create_access_point", IDLValue.create(accessPointRequest, new PojoSerializer()));
      assertTrue(false);
    } catch (Exception e) {
      assertTrue(e.getMessage().contains("could not be authenticated"));
    }
  }

  @Test(priority = 40)
  @SneakyThrows
  public void createAccessPoint_whenOk() {    
    var accessPointRequest = new HTTPAccessPointRequest(identity.derEncodedPublickey, "icon", "device", "browser");
    var response = callUpdateHttp("create_access_point", IDLValue.create(accessPointRequest, new PojoSerializer()));

    assertEquals(200, response.statusCode.intValue());
    assertTrue(response.error.isEmpty());
    assertTrue(response.data.isPresent());
  }

  @Test(priority = 50)
  @SneakyThrows
  public void recoverAccount_whenOk() {
    var response = callUpdateHttp("recover_account", IDLValue.create(10000l, Type.NAT64));

    assertEquals(200, response.statusCode.intValue());
    assertTrue(response.error.isEmpty());
    assertTrue(response.data.isPresent());
  }

  @Test(priority = 60)
  public void recoverAccount_whenError() {
    try {
      callUpdateHttp("recover_account", IDLValue.create(10002l, Type.NAT64));
      assertTrue(false);
    } catch (Exception e) {
      assertTrue(e.getMessage().contains("could not be authenticated"));
    }
  }

  @SneakyThrows
  private Long register() {
    var challengeResponse = iiProxy.createChallenge().get();

    var challengeResult = new ChallengeResult();
    challengeResult.challengeKey = challengeResponse.challengeKey;
    challengeResult.chars = "a";

    var deviceData = new DeviceData();
    deviceData.pubkey = identity.derEncodedPublickey;
    deviceData.alias = "Device1";
    deviceData.purpose = Purpose.authentication;
    deviceData.keyType = KeyType.platform;

    var registerResult = iiProxy.register(deviceData, challengeResult).get();

    return registerResult.registeredValue.userNumber;
  }

  @SneakyThrows
  private HttpResponse<?> callUpdateHttp(String methodName, IDLValue... idls) {
      var response = UpdateBuilder
              .create(agent, im, methodName)
              .arg(IDLArgs.create(Arrays.asList(idls)).toBytes())
              .callAndWait(Waiter.create(120, 2));
      return IDLArgs.fromBytes(response.get())
              .getArgs()
              .get(0)
              .getValue(PojoDeserializer.create(), HttpResponse.class);
  }

}
