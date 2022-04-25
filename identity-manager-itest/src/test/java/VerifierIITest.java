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
//            call("common/dfx_stop");
//            callDfxCommand("rm -rf .dfx");
            call("common/use_default_persona");
            ROOT_IDENTITY = call("common/get_principal").trim();
//            call("common/init_dfx_project");
//            callDfxCommand(String.format(getScript("common/deploy_project").trim(), "verifier"));
//            callDfxCommand(String.format(getScript("common/deploy_project").trim(), "identity_manager"));
            String im = call("common/get_canister_id", "identity_manager").trim();
            String verifier_id = call("common/get_canister_id", "verifier").trim();
            identity_manager = call("common/configure_dfx_project", "identity_manager", ROOT_IDENTITY, TTL, TTL_REFRESH, WHITELISTED_PHONE_NUMBERS, DISABLED_HEARTBEAT, BACKUP_CANISTER_ID, verifier_id);
            verifier = call("common/configure_verifier", im);

            if (++i >= DEFAULT_TRIES)
                System.exit(1);

        } while (identity_manager.isEmpty() || verifier.isEmpty());
    }

    @Test(priority = 10)
    public void init_user() {
        var phoneNumber = "+380991111111";
        var phoneNumberSha2 = "+380991111111_SHA2";
        call("request/create_account", "12345");
        command("request/post_token", phoneNumber, phoneNumberSha2, TOKEN, ROOT_IDENTITY);
        command("request/verify_token", TOKEN);
        call("account/req_get_account", "identity_manager");
        var actual = command("account/req_get_pn_sha2", "identity_manager", ROOT_IDENTITY);
        assertEquals("(record{data=null;error=opt\"Nopersonawithsuchdomain\";status_code=404:nat16;},)", actual);
    }

    @Test(priority = 11)
    public void no_such_domain() {
        String init_certificate = "dfx canister call verifier generate_pn_token '(\"TEST_DOMAIN\")'";
        String key = callDfxCommand(init_certificate);
        key = key.replaceAll(",", "");
        call("persona/req_create_persona");
        String upgrade = String.format("dfx canister call verifier resolve_token '(%s)'", key);
        String call_certificate = callDfxCommand(upgrade);
        assertTrue(call_certificate.contains("client_principal = \"sculj-2sjuf-dxqlm-dcv5y-hin5x-zfyvr-tzngf-bt5b5-dwhcc-zbsqf-rae"));
        assertTrue(call_certificate.contains("phone_number_sha2 = opt \"+380991111111_SHA2\";"));
        assertTrue(call_certificate.contains("domain = \"TEST_DOMAIN\";\n"));
    }


    @Test(priority = 30)
    public void is_owner() {
        String init_certificate = "dfx canister call verifier is_phone_number_approved '(\"sculj-2sjuf-dxqlm-dcv5y-hin5x-zfyvr-tzngf-bt5b5-dwhcc-zbsqf-rae\")'";
        String key = callDfxCommand(init_certificate);
        assertEquals("(record { data = opt true; error = null; status_code = 200 : nat16 })\n", key);
    }

}
