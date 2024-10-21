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

    @Test(priority = 50)
    public void createAccountSameAnchorExpectError() {
        call("request/post_token", PHONE, PHONE_SHA2, TOKEN, ROOT_IDENTITY);
        String actual = call("account/req_create_exist_account");
        validateWithFormatIdentity("account/exp_anchor_exists", actual);
    }

    @Test(priority = 52)
    public void recoverAccountExpectAccount() {
        String actual = call("account/req_recover_account");
        validateWithFormatIdentity("account/exp_account", actual);
    }

    @Test(priority = 70)
    @Ignore //timestamp
    public void getLogsExpectSuccess() {
        validateWithFormatIdentity("account/exp_logs", call("common/req_get_logs"));
        validateWithFormatIdentity("account/exp_all_logs", call("common/req_get_all_logs"));
    }

}
