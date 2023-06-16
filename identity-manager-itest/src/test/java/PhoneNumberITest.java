import org.testng.annotations.Ignore;
import org.testng.annotations.Test;

import static org.testng.Assert.assertEquals;

@Ignore
public class PhoneNumberITest extends BaseIdentityManagerITest {

    @Override
    String getHeartBeatPeriod() {
        return "100";
    }

    @Test(priority = 11)
    public void validatePhoneWhenNotLambdaUser() {
        command("common/use_test_persona");
        var actual = command("request/validate_phone", ROOT_IDENTITY, PHONE);
        var expected = get("response/response", "opt \"Unauthorized.\"", "403");
        assertEquals(actual, expected);
        command("common/use_default_persona");
    }

    @Test(priority = 12)
    public void postTokenWhenNotLambdaUser() {
        command("common/use_test_persona");
        var actual = command("request/post_token", PHONE, PHONE_SHA2, TOKEN, ROOT_IDENTITY);
        var expected = get("response/response", "opt \"Unauthorized.\"", "403");
        assertEquals(actual, expected);
        command("common/use_default_persona");
    }

    @Test(priority = 13)
    public void validatePhoneWhenAnonymousUser() {
        var actual = command("request/validate_phone", "aaaa", PHONE_SHA2);
        var expected = get("response/response_multiline", "opt \"Anonymous user is forbidden.\"", "403");
        assertEquals(actual, expected);
    }

    @Test(priority = 14)
    public void postTokenWhenAnonymousUser() {
        var actual = command("request/post_token", PHONE, PHONE_SHA2, TOKEN, "aaaa");
        var expected = get("response/response_multiline", "opt \"Anonymous user is forbidden.\"", "403");
        assertEquals(actual, expected);
    }

    @Test(priority = 15)
    public void validatePhoneWhenAccountNotFound() {
        var actual = command("request/validate_phone", ROOT_IDENTITY, PHONE_SHA2);
        var expected = get("response/response", "opt \"Account not found.\"", "404");
        assertEquals(actual, expected);
    }

    @Test(priority = 16)
    public void postTokenWhenAccountNotFound() {
        var actual = command("request/post_token", PHONE, PHONE_SHA2, TOKEN, ROOT_IDENTITY);
        var expected = get("response/response", "opt \"Account not found.\"", "404");
        assertEquals(actual, expected);
    }

    @Test(priority = 17)
    public void verifyTokenWhenAccountNotFound() {
        var actual = command("request/verify_token", TOKEN);
        var expected = get("response/response", "opt \"Account not found.\"", "404");
        assertEquals(actual, expected);
    }

    @Test(priority = 21)
    public void validatePhoneWhenPhoneNumberNotExist() {
        command("request/create_account", ANCHOR);
        var actual = command("request/validate_phone", ROOT_IDENTITY, PHONE_SHA2);
        var expected = get("response/response", "null", "200");
        assertEquals(actual, expected);
    }

    @Test(priority = 22)
    public void validatePhoneWhenPhoneNumberNotExistByAccessPoint() {
        String accessPoint = call("device/req_create_access_point", ROOT_IDENTITY);
        String principle = TestUtils.cutField(accessPoint, "principal_id").second().split("\"")[1];
        var actual = command("request/validate_phone", principle, PHONE_SHA2);
        System.out.println(actual);
        var expected = get("response/response", "null", "200");
        assertEquals(actual, expected);
    }

    @Test(priority = 31)
    public void postTokenWhenPhoneNumberNotExist() {
        var actual = command("request/post_token", PHONE, PHONE_SHA2, TOKEN, ROOT_IDENTITY);
        var expected = get("response/response", "null", "200");
        assertEquals(actual, expected);
    }

    @Test(priority = 41)
    public void verifyTokenWhenTokenNotMatch() {
        var actual = command("request/verify_token", "123");
        var expected = get("response/response_error_no_data", "Incorrect verification code, please try again.", "400");
        assertEquals(actual, expected);
    }

    @Test(priority = 51)
    public void verifyTokenWhenOk() {
        var actual = command("request/verify_token", TOKEN);
        var expected = get("response/response", "null", "200");
        assertEquals(actual, expected);
    }
}
