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

}
