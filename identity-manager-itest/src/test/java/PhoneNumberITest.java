import lombok.SneakyThrows;
import org.testng.annotations.Ignore;
import org.testng.annotations.Test;

public class PhoneNumberITest extends BaseIdentityManagerITest {

    @Test(priority = 1)
    public void postTokenExceptCorrectResponse() {
        var actual = call("phone-number/req_post_token");
        validateWithFormatIdentity("phone-number/exp_post_token", actual);
    }

    @Test(priority = 2)
    public void createAccountExpectPhoneNumberNotFound() {
        var actual = call("phone-number/req_create_account_incorrect_phone_number");
        validateWithFormatIdentity("phone-number/exp_create_account_incorrect_phone_number", actual);
    }

    @Test(priority = 3)
    public void createAccountExpectTokenIsIncorrect() {
        var actual = call("phone-number/req_create_account_incorrect_token");
        validateWithFormatIdentity("phone-number/exp_create_account_incorrect_token", actual);
    }

    @Test(priority = 4)
    public void validatePhoneNumberExpectTrueWhenPhoneNumberNotExists() {
        var actual = call("phone-number/req_validate_phone_number");
        validateWithFormatIdentity("phone-number/exp_true_validate_phone_number", actual);
    }

    @Test(priority = 5)
    public void validatePhoneNumberExpectFalseWhenPhoneNumberExists() {
        call("account/req_create_account");
        var actual = call("phone-number/req_validate_phone_number");
        validateWithFormatIdentity("phone-number/exp_false_validate_phone_number", actual);
    }

    @SneakyThrows
    @Test(priority = 6)
    public void switchPersonaAndGetRootAccount() {
        call("common/create_test_persona");
        call("common/use_test_persona");

        var actual = call("phone-number/req_post_token");
        validateWithFormatIdentity("phone-number/exp_post_token_unauthorized", actual);
    }

    @Ignore
    @SneakyThrows
    @Test(priority = 7)
    public void createAccountExpectPhoneNumberNotFoundByTokenExpiration() {
        call("common/use_default_persona");
        call("phone-number/req_post_token_default");

        Thread.sleep(11000);

        var actual = call("account/req_create_account");
        validateWithFormatIdentity("phone-number/exp_create_account_incorrect_phone_number", actual);
    }

}
