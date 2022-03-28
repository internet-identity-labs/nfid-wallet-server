import org.testng.annotations.Test;

import static org.testng.Assert.assertEquals;

public class PhoneNumberITest extends BaseIdentityManagerITest {

    @Override
    String getHeartBeatPeriod() {
        return "100";
    }

    @Test(priority = 1)
    public void validatePhoneWhenNotLambdaUser() {
        command("common/use_test_persona");
        var actual = command("request/validate_phone", ROOT_IDENTITY, PHONE);
        var expected = get("response/response", "opt \"Unauthorized.\"", "403");
        assertEquals(actual, expected);
        command("common/use_default_persona");
    }

    @Test(priority = 1)
    public void postTokenWhenNotLambdaUser() {
        command("common/use_test_persona");
        var actual = command("request/post_token", PHONE, TOKEN, ROOT_IDENTITY);
        var expected = get("response/response", "opt \"Unauthorized.\"", "403");
        assertEquals(actual, expected);
        command("common/use_default_persona");
    }

    @Test(priority = 1)
    public void validatePhoneWhenAnonymousUser() {
        var actual = command("request/validate_phone", "aaaa", PHONE);
        var expected = get("response/response_multiline", "opt \"Anonymous user is forbidden.\"", "403");
        assertEquals(actual, expected);
    }

    @Test(priority = 1)
    public void postTokenWhenAnonymousUser() {
        var actual = command("request/post_token", PHONE, TOKEN, "aaaa");
        var expected = get("response/response_multiline", "opt \"Anonymous user is forbidden.\"", "403");
        assertEquals(actual, expected);
    }

    @Test(priority = 1)
    public void validatePhoneWhenAccountNotFound() {
        var actual = command("request/validate_phone", ROOT_IDENTITY, PHONE);
        var expected = get("response/response", "opt \"Account not found.\"", "404");
        assertEquals(actual, expected);
    }

    @Test(priority = 1)
    public void postTokenWhenAccountNotFound() {
        var actual = command("request/post_token", PHONE, TOKEN, ROOT_IDENTITY);
        var expected = get("response/response", "opt \"Account not found.\"", "404");
        assertEquals(actual, expected);
    }

    @Test(priority = 1)
    public void verifyTokenWhenAccountNotFound() {
        var actual = command("request/verify_token", TOKEN);
        var expected = get("response/response", "opt \"Account not found.\"", "404");
        assertEquals(actual, expected);
    }

    @Test(priority = 2)
    public void validatePhoneWhenPhoneNumberNotExist() {
        command("request/create_account", ANCHOR);
        var actual = command("request/validate_phone", ROOT_IDENTITY, PHONE);
        var expected = get("response/response", "null", "200");
        assertEquals(actual, expected);
    }

    @Test(priority = 3)
    public void postTokenWhenPhoneNumberNotExist() {
        var actual = command("request/post_token", PHONE, TOKEN, ROOT_IDENTITY);
        var expected = get("response/response", "null", "200");
        assertEquals(actual, expected);
    }

    @Test(priority = 4)
    public void verifyTokenWhenTokenNotMatch() {
        var actual = command("request/verify_token", "123");
        var expected = get("response/response", "opt \"Token does not match.\"", "400");
        assertEquals(actual, expected);
    }

    @Test(priority = 5)
    public void verifyTokenWhenOk() {
        var actual = command("request/verify_token", TOKEN);
        var expected = get("response/response", "null", "200");
        assertEquals(actual, expected);
    }

    @Test(priority = 6)
    public void validatePhoneWhenTooManyRequests() {
        var actual = command("request/validate_phone", ROOT_IDENTITY, PHONE);
        var expected = get("response/response", "opt \"Too many requests.\"", "429");
        assertEquals(actual, expected);
    }

    @Test(priority = 7)
    public void validatePhoneNumberExists() throws InterruptedException {
        command("common/configure_dfx_project", "identity_manager", KEY, ROOT_IDENTITY, TTL, "1", WHITELISTED_PHONE_NUMBERS, 0, BACKUP_CANISTER_ID, BACKUP_CANISTER_ID);
        command("request/post_token", PHONE, TOKEN, ROOT_IDENTITY);

        Thread.sleep(1000);

        var actual = command("request/validate_phone", ROOT_IDENTITY, PHONE);
        var expected = get("response/response", "null", "204");
        assertEquals(actual, expected);
    }

    @Test(priority = 7)
    public void validatePhoneNumberNotExists() throws InterruptedException {
        var phoneNumber = "+380991111111";

        command("request/post_token", phoneNumber, TOKEN, ROOT_IDENTITY);

        Thread.sleep(1000);

        var actual = command("request/validate_phone", ROOT_IDENTITY, phoneNumber);
        var expected = get("response/response", "null", "200");
        assertEquals(actual, expected);
    }

    @Test(priority = 8)
    public void verifyTokenWhenPrincipalIdNotExists() throws InterruptedException {
        command("common/configure_dfx_project", "identity_manager", KEY, ROOT_IDENTITY, "1", TTL_REFRESH, WHITELISTED_PHONE_NUMBERS, 0, BACKUP_CANISTER_ID, BACKUP_CANISTER_ID);
        command("request/post_token", PHONE, TOKEN, ROOT_IDENTITY);

        Thread.sleep(1000);

        var actual = command("request/verify_token", TOKEN);
        var expected = get("response/response", "opt \"Principal id not found.\"", "404");
        assertEquals(actual, expected);
    }

}
