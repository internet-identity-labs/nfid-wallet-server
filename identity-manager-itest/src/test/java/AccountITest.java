import org.testng.annotations.Test;


public class AccountITest extends BaseIdentityManagerITest {

    @Test()
    public void createAccountWithInvalidNameExpectErrorResponse() {
        String actual = call("account/req_create_invalid_name_account");
        validateWithFormatIdentity("account/exp_account_invalid_name", actual);
    }

    @Test(priority = 1)
    public void createAccountExpectCorrectResponse() {
        String actual = call("account/req_create_account");
        validateWithFormatIdentity("account/exp_account", actual);
    }

    @Test(priority = 2)
    public void getAccountCreatedInPreviousTest() {
        String actual = call("account/req_get_account");
        validateWithFormatIdentity("account/exp_account", actual);
    }

    @Test(priority = 3)
    public void updateAccountNameExpectUpdated() {
        String actual = call("account/req_update_account_name");
        validateWithFormatIdentity("account/exp_account_upd_name", actual);
    }

    @Test(priority = 4)
    public void createAccountSecondTimeExpectPhoneNumberExists() {
        String actual = call("account/req_create_account");
        validateWithFormatIdentity("account/exp_phone_number_exists", actual);
    }

}
