import lombok.SneakyThrows;
import org.testng.Assert;
import org.testng.annotations.Ignore;
import org.testng.annotations.Test;


public class AccountITest extends BaseIdentityManagerITest {

    @Test(priority = 10)
    public void createAccountExpectCorrectResponse() {
        var actual = call("request/create_account", "12345");
        validateWithFormatIdentity("account/exp_account", actual);
    }

    @Test(priority = 20)
    public void getAccountCreatedInPreviousTest() {
        String actual = call("account/req_get_account", "identity_manager");
        validateWithFormatIdentity("account/exp_account", actual);
    }

    @Test(priority = 21)
    public void getAccountCreatedInPreviousTestByPrincipal() {
        String actual = call("account/req_get_account_by_principal", "identity_manager", ROOT_IDENTITY);
        validateWithFormatIdentity("account/exp_account", actual);
    }

    @Test(priority = 21)
    public void getAccountPNSha2CreatedInPreviousTest() {
        var actual = command("account/req_get_pn_sha2", "identity_manager", ROOT_IDENTITY);
        var expected = get("response/response_error", "Phone number not verified", "404");
        Assert.assertEquals(actual, expected);
    }

    @Test(priority = 30)
    public void updateAccountNameExpectUpdated() {
        String actual = call("account/req_update_account_name");
        validateWithFormatIdentity("account/exp_account_upd_name", actual);
    }

    @Test(priority = 50)
    public void createAccountSameAnchorExpectError() {
        call("request/post_token", PHONE, PHONE_SHA2, TOKEN, ROOT_IDENTITY);
        String actual = call("account/req_create_exist_account");
        validateWithFormatIdentity("account/exp_anchor_exists", actual);
    }

    @SneakyThrows
    @Test(priority = 51)
    public void replicateAccountExpectCopyOfAccountByHeartbeat() {
        Thread.sleep(2000);
        String actual = call("account/req_get_account", "identity_manager_replica");
        validateWithFormatIdentity("account/exp_account_upd_name", actual);
    }

    @Test(priority = 52)
    public void recoverAccountExpectAccount() {
        String actual = call("account/req_recover_account");
        validateWithFormatIdentity("account/exp_account_upd_name", actual);
    }

    @SneakyThrows
    @Test(priority = 53)
    public void restoreAccountExpectCopeByApiCall() {
        validateWithFormatIdentity("common/resp_bool_success", call("account/req_remove_account", "identity_manager_replica"));
        String actual = call("account/req_get_account", "identity_manager_replica");
        validateWithFormatIdentity("account/exp_unable_to_find_acc", actual);
        call("account/req_restore_account", "identity_manager", BACKUP_CANISTER_ID);
        actual = call("account/req_get_account", "identity_manager_replica");
        validateWithFormatIdentity("account/exp_account_upd_name", actual);
    }

    @Test(priority = 60)
    public void removeAccountExpectSuccess() {
        validateWithFormatIdentity("common/resp_bool_success", call("account/req_remove_account", "identity_manager"));
        validateWithFormatIdentity("account/exp_unable_to_remove_account", call("account/req_remove_account", "identity_manager"));
        String actual = call("account/req_create_account");
        validateWithFormatIdentity("account/exp_account", actual);
    }

    @Test(priority = 70)
    @Ignore //timestamp
    public void getLogsExpectSuccess() {
        validateWithFormatIdentity("account/exp_logs", call("common/req_get_logs"));
        validateWithFormatIdentity("account/exp_all_logs", call("common/req_get_all_logs"));
    }

}
