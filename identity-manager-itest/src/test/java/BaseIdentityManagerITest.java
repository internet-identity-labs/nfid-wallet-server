import org.testng.annotations.BeforeClass;

public class BaseIdentityManagerITest extends BaseDFXITest {
    private final static int DEFAULT_TRIES = 5;

    @BeforeClass
    public void initDfxProject() {
        int i = 0;
        String identity_manager;
        String identity_manager_replica;
        do {
            call("common/dfx_stop");
            callDfxCommand("rm -rf .dfx");
            call("common/create_test_persona");
            call("common/use_default_persona");
            ROOT_IDENTITY = call("common/get_principal").trim();
            call("common/init_dfx_project");
            var command = String.format(getScript("common/deploy_project").trim(), "identity_manager");
            callDfxCommand(command);
            String im = call("common/get_canister_id", "identity_manager").trim();
            identity_manager = call("common/configure_dfx_project", "identity_manager", ROOT_IDENTITY, ROOT_IDENTITY, TTL, TTL_REFRESH, WHITELISTED_PHONE_NUMBERS,  im);
            if (++i >= DEFAULT_TRIES) {
                call("common/dfx_stop");
                System.exit(1);
            }

        } while (identity_manager.isEmpty());
        call("request/post_token", PHONE, PHONE_SHA2, TOKEN, ROOT_IDENTITY);
    }

    String getHeartBeatPeriod() {
        return HEARTBEAT_PERIOD;
    }

}
