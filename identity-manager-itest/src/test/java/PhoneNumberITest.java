import org.testng.annotations.Test;

import static org.testng.Assert.assertEquals;

public class PhoneNumberITest extends BaseIdentityManagerITest {

    @Test(priority = 1)
    public void validatePhoneWhenNotLambdaUser() {
        call("common/use_test_persona");
        var actual = call("request/validate_phone", ROOT_IDENTITY, PHONE);
        var expected = get("response/response", "opt \"Unauthorized.\"", "403");
        assertEquals(actual, expected);
        call("common/use_default_persona");
    }

    @Test(priority = 1)
    public void postTokenWhenNotLambdaUser() {
        call("common/use_test_persona");
        var actual = call("request/post_token", PHONE, TOKEN, ROOT_IDENTITY);
        var expected = get("response/response", "opt \"Unauthorized.\"", "403");
        assertEquals(actual, expected);
        call("common/use_default_persona");
    }

    @Test(priority = 1)
    public void validatePhoneWhenAnonymousUser() {
        var actual = call("request/validate_phone", "aaaa", PHONE);
        var expected = get("response/response_multiline", "opt \"Anonymous user is forbidden.\"", "403");
        assertEquals(actual, expected);
    }

    @Test(priority = 1)
    public void postTokenWhenAnonymousUser() {
        var actual = call("request/post_token", PHONE, TOKEN, "aaaa");
        var expected = get("response/response_multiline", "opt \"Anonymous user is forbidden.\"", "403");
        assertEquals(actual, expected);
    }

    @Test(priority = 1)
    public void validatePhoneWhenAccountNotFound() {
        var actual = call("request/validate_phone", ROOT_IDENTITY, PHONE);
        var expected = get("response/response", "opt \"Account not found.\"", "404");
        assertEquals(actual, expected);
    }

    @Test(priority = 1)
    public void postTokenWhenAccountNotFound() {
        var actual = call("request/post_token", PHONE, TOKEN, ROOT_IDENTITY);
        var expected = get("response/response", "opt \"Account not found.\"", "404");
        assertEquals(actual, expected);
    }

    @Test(priority = 1)
    public void verifyTokenWhenAccountNotFound() {
        var actual = call("request/verify_token", TOKEN);
        var expected = get("response/response", "opt \"Account not found.\"", "404");
        assertEquals(actual, expected);
    }

    @Test(priority = 2)
    public void validatePhoneWhenPhoneNumberNotExist() {
        call("request/create_account", ANCHOR);
        var actual = call("request/validate_phone", ROOT_IDENTITY, PHONE);
        var expected = get("response/response", "null", "200");
        assertEquals(actual, expected);
    }

    @Test(priority = 3)
    public void postTokenWhenPhoneNumberNotExist() {
        var actual = call("request/post_token", PHONE, TOKEN, ROOT_IDENTITY);
        var expected = get("response/response", "null", "200");
        assertEquals(actual, expected);
    }

    @Test(priority = 4)
    public void verifyTokenWhenTokenNotMatch() {
        var actual = call("request/verify_token", "123");
        var expected = get("response/response", "opt \"Token does not match.\"", "400");
        assertEquals(actual, expected);
    }

    @Test(priority = 5)
    public void verifyTokenWhenOk() {
        var actual = call("request/verify_token", TOKEN);
        var expected = get("response/response", "null", "200");
        assertEquals(actual, expected);
    }

    @Test(priority = 6)
    public void validatePhoneWhenTooManyRequests() {
        var actual = call("request/validate_phone", ROOT_IDENTITY, PHONE);
        var expected = get("response/response", "opt \"Too many requests.\"", "429");
        assertEquals(actual, expected);
    }

    @Test(priority = 7)
    public void validatePhoneNumberExists() throws InterruptedException {
        call("common/configure_dfx_project", KEY, ROOT_IDENTITY, TTL, "1", WHITELISTED_PHONE_NUMBERS);
        call("request/post_token", PHONE, TOKEN, ROOT_IDENTITY);

        Thread.sleep(1000);

        var actual = call("request/validate_phone", ROOT_IDENTITY, PHONE);
        var expected = get("response/response", "null", "204");
        assertEquals(actual, expected);
    }

    @Test(priority = 7)
    public void validatePhoneNumberNotExists() throws InterruptedException {
        var phoneNumber = "+380991111111";

        call("request/post_token", phoneNumber, TOKEN, ROOT_IDENTITY);

        Thread.sleep(1000);

        var actual = call("request/validate_phone", ROOT_IDENTITY, phoneNumber);
        var expected = get("response/response", "null", "200");
        assertEquals(actual, expected);
    }

    @Test(priority = 8)
    public void verifyTokenWhenPrincipalIdNotExists() throws InterruptedException {
        call("common/configure_dfx_project", KEY, ROOT_IDENTITY, "1", TTL_REFRESH, WHITELISTED_PHONE_NUMBERS);
        call("request/post_token", PHONE, TOKEN, ROOT_IDENTITY);

        Thread.sleep(1000);

        var actual = call("request/verify_token", TOKEN);
        var expected = get("response/response", "opt \"Principal id not found.\"", "404");
        assertEquals(actual, expected);
    }

}
