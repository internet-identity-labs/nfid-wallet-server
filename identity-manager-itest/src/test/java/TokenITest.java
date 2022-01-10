import lombok.SneakyThrows;
import org.testng.annotations.Ignore;
import org.testng.annotations.Test;

public class TokenITest extends BaseDFXITest {

    @Test(priority = 1)
    public void postTokenExceptCorrectResponse() {
        var actual = call("token/req_post_token");
        validateWithFormatIdentity("token/exp_post_token", actual);
    }

    @Test(priority = 2)
    public void createAccountExpectPhoneNumberNotFound() {
        var actual = call("token/req_create_account_incorrect_phone_number");
        validateWithFormatIdentity("token/exp_create_account_incorrect_phone_number", actual);
    }

    @Test(priority = 3)
    public void createAccountExpectTokenIsIncorrect() {
        var actual = call("token/req_create_account_incorrect_token");
        validateWithFormatIdentity("token/exp_create_account_incorrect_token", actual);
    }

    @SneakyThrows
    @Test(priority = 4)
    public void switchPersonaAndGetRootAccount() {
        call("common/create_test_persona");
        call("common/use_test_persona");

        var actual = call("token/req_post_token");
        validateWithFormatIdentity("token/exp_post_token_unauthorized", actual);
    }

    @Ignore
    @SneakyThrows
    @Test(priority = 5)
    public void createAccountExpectPhoneNumberNotFoundByTokenExpiration() {
        call("common/use_default_persona");
        call("token/req_post_token_default");

        Thread.sleep(11000);

        var actual = call("account/req_create_account");
        validateWithFormatIdentity("token/exp_create_account_incorrect_phone_number", actual);
    }

}
