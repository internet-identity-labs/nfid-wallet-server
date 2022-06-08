import lombok.SneakyThrows;
import lombok.extern.slf4j.Slf4j;
import org.ic4j.agent.*;
import org.ic4j.agent.http.ReplicaJavaHttpTransport;
import org.ic4j.agent.identity.BasicIdentity;
import org.ic4j.agent.identity.Identity;
import org.ic4j.candid.parser.IDLArgs;
import org.ic4j.candid.parser.IDLValue;
import org.ic4j.candid.pojo.PojoDeserializer;
import org.ic4j.candid.pojo.PojoSerializer;
import org.ic4j.types.Principal;
import org.testng.annotations.BeforeClass;
import org.testng.annotations.Test;
import records.Application;
import records.HttpResponse;

import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

import static constants.Constants.STATUS_NOT_FOUND;
import static constants.Constants.STATUS_SUCCESS;
import static org.testng.AssertJUnit.*;

@Slf4j
public class ApplicationITest extends BaseDFXITest {

    private Agent agent;
    private Principal canister;

    @SneakyThrows
    @BeforeClass
    public void init() {
        int i = 0;
        String identity_manager;
        do {
            log.info("Initialising ApplicationITest");
            call("common/dfx_stop");
            callDfxCommand("rm -rf .dfx");
            call("common/create_test_persona");
            call("common/use_admin_persona");
            ROOT_IDENTITY = call("common/get_principal").trim();
            call("common/init_dfx_project_full");
            call("common/deploy_im");

            String icLocation = "http://localhost:8000";
            String icCanister = call("common/get_canister_id", "identity_manager").trim();
            Path path = Paths.get(this.getClass().getClassLoader().getResource("identity/identity.pem").getPath());
            Identity identity = BasicIdentity.fromPEMFile(path);

            identity_manager = callDfxCommand(String.format("dfx canister call %s configure '(record {env = opt \"test\";})'", "identity_manager"));

            ReplicaTransport transport = ReplicaJavaHttpTransport.create(icLocation);
            agent = new AgentBuilder().transport(transport).identity(identity).build();
            canister = Principal.fromString(icCanister);
            if (++i >= 5) {
                call("common/dfx_stop");
                log.error("Stopping ApplicationITest");
                System.exit(1);
            }
        } while (identity_manager.isEmpty());

    }

    @Test(priority = 10)
    public void createApplicationExpectCorrectResponse() {
        Application application = application("dom", "TEST_APP", 1);
        HttpResponse httpResponse = callUpdateHttp(application, "create_application");
        List<Application> applications = httpResponse.getVectorFromData(Application.class);
        assertEquals(STATUS_SUCCESS, httpResponse.statusCode);
        assertEquals(Optional.empty(), httpResponse.error);
        assertEquals(1, applications.size());
        assertEquals(1, applications.get(0).userLimit.intValue());
        assertEquals("dom", applications.get(0).domain);
        assertEquals("TEST_APP", applications.get(0).name);
    }

    @Test(priority = 20)
    public void createApplicationWithSameNameExpectErrorResponse() {
        Application application = application("dom2", "TEST_APP", 1);
        HttpResponse httpResponse = callUpdateHttp(application, "create_application");
        assertEquals(STATUS_NOT_FOUND, httpResponse.statusCode);
        assertEquals(Optional.empty(), httpResponse.data);
        assertEquals("Unable to create Application. Application exists", httpResponse.error.get());
    }

    @Test(priority = 30)
    public void createApplicationWithSameDomExpectCorrectResponse() {
        Application application = application("dom2", "TEST_APP2", 1);
        HttpResponse httpResponse = callUpdateHttp(application, "create_application");
        List<Application> applications = httpResponse.getVectorFromData(Application.class);
        assertEquals(STATUS_SUCCESS, httpResponse.statusCode);
        assertEquals(Optional.empty(), httpResponse.error);
        assertEquals(2, applications.size());
        assertEquals(1, applications.get(0).userLimit.intValue());
        assertEquals("dom", applications.get(0).domain);
        assertEquals("TEST_APP", applications.get(0).name);
        assertEquals(1, applications.get(1).userLimit.intValue());
        assertEquals("dom2", applications.get(1).domain);
        assertEquals("TEST_APP2", applications.get(1).name);
    }

    @Test(priority = 40)
    public void readApplicationsExpectCorrectResponse() {
        HttpResponse httpResponse = callQueryHttp("read_applications", Optional.empty());
        List<Application> applications = httpResponse.getVectorFromData(Application.class);
        assertEquals(STATUS_SUCCESS, httpResponse.statusCode);
        assertEquals(Optional.empty(), httpResponse.error);
        assertEquals(2, applications.size());
        assertEquals(1, applications.get(0).userLimit.intValue());
        assertEquals("dom", applications.get(0).domain);
        assertEquals("TEST_APP", applications.get(0).name);
        assertEquals(1, applications.get(1).userLimit.intValue());
        assertEquals("dom2", applications.get(1).domain);
        assertEquals("TEST_APP2", applications.get(1).name);
    }

    @Test(priority = 50)
    public void isOverLimitExpectCorrectResponse() {
        call("account/req_create_account"); //TODO migrate to new flow when account and persona classes ready
        call("persona/req_create_persona");
        call("persona/req_create_persona_2");
        validateWithFormatIdentity("persona/exp_under_limit_for_app", call("application/req_is_over_limit"));
        call("application/req_create_application_over_limit");
        validateWithFormatIdentity("persona/exp_over_limit_for_app", call("application/req_is_over_limit"));
    }

    @Test(priority = 60)
    public void deleteApplicationIsOverLimitExpectCorrectResponse() {
        HttpResponse httpResponse = callUpdateHttp("TEST_PERSONA", "delete_application");
        assertTrue((Boolean) httpResponse.data.get());
        assertEquals(STATUS_SUCCESS, httpResponse.statusCode);
        assertEquals(Optional.empty(), httpResponse.error);
        httpResponse = callQueryHttp("is_over_the_application_limit", Optional.of("TEST_DOMAIN"));
        assertFalse((Boolean) httpResponse.data.get());
        assertEquals(STATUS_SUCCESS, httpResponse.statusCode);
        assertEquals(Optional.empty(), httpResponse.error);
    }

    @Test(priority = 61)
    public void isOverDefaultLimitExpectCorrectResponse() {
        call("persona/req_create_persona"); //TODO migrate to new flow when account and persona classes ready
        call("persona/req_create_persona");
        call("persona/req_create_persona");
        call("persona/req_create_persona");
        call("persona/req_create_persona");
        validateWithFormatIdentity("persona/exp_create_persona_over_limit_domain",  call("persona/req_create_persona"));
        call("application/req_create_application_over_limit");
        validateWithFormatIdentity("persona/exp_over_limit_for_app", call("application/req_is_over_limit"));
    }

    private Application application(String domain, String name, int userLimit) {
        return Application.builder()
                .domain(domain)
                .name(name)
                .userLimit((short) userLimit)
                .build();
    }

    @SneakyThrows
    private HttpResponse callUpdateHttp(Object application, String methodName) {
        IDLValue idlValue = IDLValue.create(application, new PojoSerializer());
        List<IDLValue> idlArgs = new ArrayList<IDLValue>();
        idlArgs.add(idlValue);
        byte[] buf = IDLArgs.create(idlArgs).toBytes();
        var response = UpdateBuilder
                .create(agent, canister, methodName)
                .arg(buf)
                .callAndWait(Waiter.create(5, 2));
        byte[] output = response.get();
        return IDLArgs.fromBytes(output).getArgs().get(0)
                .getValue(PojoDeserializer.create(), HttpResponse.class);
    }

    @SneakyThrows
    private HttpResponse callQueryHttp(String methodName, Optional<Object> params) {
        IDLValue idlValue = params.isEmpty() ?
                IDLValue.create(new PojoSerializer())
                : IDLValue.create(params.get(), new PojoSerializer());

        List<IDLValue> idlArgs = new ArrayList<IDLValue>();
        idlArgs.add(idlValue);
        byte[] buf = IDLArgs.create(idlArgs).toBytes();
        var response = QueryBuilder.
                create(agent, canister, methodName)
                .arg(buf)
                .call();
        byte[] output = response.get();
        return IDLArgs.fromBytes(output).getArgs().get(0)
                .getValue(PojoDeserializer.create(), HttpResponse.class);
    }
}
