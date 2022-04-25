import org.testng.annotations.BeforeClass;

public class BaseIdentityManagerITest extends BaseDFXITest {
    private final static int DEFAULT_TRIES = 20;

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
            call("common/deploy_dfx_project"); //TODO split
            BACKUP_CANISTER_ID = call("common/get_canister_id", "identity_manager_replica").trim();
            String im = call("common/get_canister_id", "identity_manager").trim();
            identity_manager = call("common/configure_dfx_project", "identity_manager", ROOT_IDENTITY, TTL, TTL_REFRESH, WHITELISTED_PHONE_NUMBERS, getHeartBeatPeriod(), BACKUP_CANISTER_ID, im);
            identity_manager_replica = call("common/configure_dfx_project", "identity_manager_replica", ROOT_IDENTITY, TTL, TTL_REFRESH, WHITELISTED_PHONE_NUMBERS, DISABLED_HEARTBEAT, BACKUP_CANISTER_ID, im);
            if (++i >= DEFAULT_TRIES)
                System.exit(1);

        } while (identity_manager.isEmpty() || identity_manager_replica.isEmpty());
        call("request/post_token", PHONE, PHONE_SHA2, TOKEN, ROOT_IDENTITY);
    }

    String getHeartBeatPeriod() {
        return HEARTBEAT_PERIOD;
    }

    ;

}
