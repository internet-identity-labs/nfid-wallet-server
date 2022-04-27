import lombok.SneakyThrows;
import org.testng.annotations.BeforeClass;
import org.testng.annotations.Test;

import static org.testng.AssertJUnit.assertEquals;
import static org.testng.AssertJUnit.assertTrue;


public class VerifierIITest extends BaseDFXITest {

    @BeforeClass
    public void initDfxProject() {
        int i = 0;
        String identity_manager;
        String verifier;
        do {
            call("common/dfx_stop");
            callDfxCommand("rm -rf .dfx");
            call("common/use_default_persona");
            ROOT_IDENTITY = call("common/get_principal").trim();
            call("common/init_dfx_project");
            callDfxCommand(String.format(getScript("common/deploy_project").trim(), "verifier"));
            callDfxCommand(String.format(getScript("common/deploy_project").trim(), "identity_manager"));
            String im = call("common/get_canister_id", "identity_manager").trim();
            String verifier_id = call("common/get_canister_id", "verifier").trim();
            identity_manager = call("common/configure_dfx_project", "identity_manager", ROOT_IDENTITY, TTL, TTL_REFRESH, WHITELISTED_PHONE_NUMBERS, DISABLED_HEARTBEAT, BACKUP_CANISTER_ID, verifier_id);
            verifier = call("common/configure_verifier", im, TTL);

            if (++i >= DEFAULT_TRIES)
                System.exit(1);

        } while (identity_manager.isEmpty() || verifier.isEmpty());
    }

    @Test(priority = 10)
    public void init_user() {
        var phoneNumber = "+380991111111";
        var phoneNumberSha2 = "+380991111111_SHA2";
        call("request/create_account", "12345");
        call("request/post_token", phoneNumber, phoneNumberSha2, TOKEN, ROOT_IDENTITY);
        call("request/verify_token", TOKEN);
    }

    @Test(priority = 11)
    public void test_phone_number_no_requested_domain_error() {
        var actual = command("account/req_get_pn_sha2", "identity_manager", ROOT_IDENTITY);
        assertEquals("(record{data=null;error=opt\"Nononcertifiedpersonawithsuchdomain\";status_code=404:nat16;},)", actual);
    }

    @Test(priority = 20)
    public void generate_and_resolve_token() {
        call("persona/req_create_persona");
        call("common/use_test_persona");
        var user = call("common/get_principal").trim();
        String init_token = "dfx canister call verifier generate_pn_token '(\"TEST_DOMAIN\")'";
        String key = callDfxCommand(init_token);
        key = key.replaceAll(",", "");
        call("common/use_default_persona");
        String call_certificate = callDfxCommand(String.format("dfx canister call verifier resolve_token '(%s)'", key));
        assertTrue(call_certificate.contains("client_principal = \"" + user));
        assertTrue(call_certificate.contains("phone_number_sha2 = opt \"+380991111111_SHA2\";"));
        assertTrue(call_certificate.contains("domain = \"TEST_DOMAIN\";\n"));
    }

    @Test(priority = 21)
    public void test_phone_number_resolved_no_requested_domain_error() {
        var actual = command("account/req_get_pn_sha2", "identity_manager", ROOT_IDENTITY);
        assertEquals("(record{data=null;error=opt\"Nononcertifiedpersonawithsuchdomain\";status_code=404:nat16;},)", actual);
    }

    @Test(priority = 30)
    public void verify_owner_of_certificate() {
        call("common/use_test_persona");
        var user = call("common/get_principal").trim();
        String init_certificate = "dfx canister call verifier is_phone_number_approved '(\"" + user + "\")'";
        String key = callDfxCommand(init_certificate);
        assertEquals("(record { data = opt true; error = null; status_code = 200 : nat16 })\n", key);
    }

    @SneakyThrows
    @Test(priority = 32)
    public void test_token_expired() {
        call("common/use_default_persona");
        String im = call("common/get_canister_id", "identity_manager").trim();
        call("common/configure_verifier", im, 1);
        call("persona/req_create_persona");
        String init_token = "dfx canister call verifier generate_pn_token '(\"TEST_DOMAIN\")'";
        String key = callDfxCommand(init_token);
        Thread.sleep(1000);
        String cmd = String.format("dfx canister call verifier resolve_token '(%s)'", key.replaceAll(",", ""));
        String call_certificate = callDfxCommand(cmd);
        assertTrue(call_certificate.isEmpty());
    }

}
