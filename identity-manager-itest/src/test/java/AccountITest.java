import org.testng.annotations.Test;


public class AccountITest extends BaseIdentityManagerITest {

    @Test()
    public void createAccountWithInvalidNameExpectErrorResponse() {
        String actual = call("account/req_create_invalid_name_account");
        validateWithFormatIdentity("account/exp_account_invalid_name", actual);
    }

    @Test(priority = 10)
    public void createAccountExpectCorrectResponse() {
        String actual = call("account/req_create_account");
        validateWithFormatIdentity("account/exp_account", actual);
    }

    @Test(priority = 22)
    public void getAccountCreatedInPreviousTest() {
        String actual = call("account/req_get_account");
        validateWithFormatIdentity("account/exp_account", actual);
    }

    @Test(priority = 30)
    public void updateAccountNameExpectUpdated() {
        String actual = call("account/req_update_account_name");
        validateWithFormatIdentity("account/exp_account_upd_name", actual);
    }

    @Test(priority = 40)
    public void createAccountSecondTimeExpectPhoneNumberExists() {
        String actual = call("account/req_create_account");
        validateWithFormatIdentity("account/exp_phone_number_exists", actual);
    }

    @Test(priority = 50)
    public void createAccountSameAnchorExpectError() {
        call("token/req_post_token_2");
        String actual = call("account/req_create_exist_account");
        validateWithFormatIdentity("account/exp_anchor_exists", actual);
    }

    @Test(priority = 60)
    public void removeAccountSameAnchorExpectError() {
        validateWithFormatIdentity("common/resp_bool_success", call("account/req_remove_account"));
        validateWithFormatIdentity("account/exp_unable_to_remove_account", call("account/req_remove_account"));
        String actual = call("account/req_create_account");
        validateWithFormatIdentity("account/exp_account", actual);
    }

}
