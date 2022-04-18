import org.testng.annotations.Test;

import static org.testng.Assert.assertEquals;

public class CredentialServiceITest extends BaseIdentityManagerITest {

    @Test(priority = 1)
    public void credentialWhenAnonymous() {
        command("request/identity_use", ANONYMOUS);

        var actual = command("request/credentials");
        var expected = get("response/data_response", "null", "opt \"Anonymous user is forbidden.\"", "403");
        assertEquals(actual, expected);
    }

    @Test(priority = 1)
    public void credentialWhenAccountNotFound() {
        command("request/identity_use", DEFAULT);

        var actual = command("request/credentials");
        var expected = get("response/data_response", "null", "opt \"Account not found.\"", "404");
        assertEquals(actual, expected);
    }

    @Test(priority = 2)
    public void credentialWhenPhoneNumberExists() {
        command("request/identity_use", DEFAULT);
        command("request/create_account", ANCHOR);
        command("request/post_token", PHONE, PHONE_SHA2, TOKEN, ROOT_IDENTITY);
        command("request/verify_token", TOKEN);

        var variant = "opt vec {variant {phone_number = record { phone_number = \"" + PHONE + "\" }};}";

        var actual = command("request/credentials");
        var expected = get("response/data_response", variant, "null", "200");
        assertEquals(actual, expected);
    }

}
