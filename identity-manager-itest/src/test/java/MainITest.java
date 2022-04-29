import org.testng.annotations.Test;

public class MainITest extends BaseIdentityManagerITest {

    @Test(priority = 1)
    public void configureWhenTheUserExpectOk() {
        var user = call("common/get_principal").trim();
        String actual = call("common/configure_dfx_project", "identity_manager", user, TTL, TTL_REFRESH, WHITELISTED_PHONE_NUMBERS, 0, BACKUP_CANISTER_ID, BACKUP_CANISTER_ID);
        validateWithFormatIdentity("main/exp_ok", actual);
    }

    @Test(priority = 2)
    public void configureWhenOtherUserExpectFail() {
        call("common/use_test_persona");
        var user = call("common/get_principal").trim();
        String actual = call("common/configure_dfx_project", "identity_manager", user, TTL, TTL_REFRESH, WHITELISTED_PHONE_NUMBERS, 0, BACKUP_CANISTER_ID, BACKUP_CANISTER_ID);
        validateWithFormatIdentity("main/exp_fail", actual);
    }

    @Test(priority = 3)
    public void getConfigExceptConfigResponse() {
        call("common/use_default_persona");
        var user = call("common/get_principal").trim();
        String actual = call("common/get_config_dfx_project", "identity_manager");
        validateWithFormatIdentity("common/exp_config", actual);
    }

}
