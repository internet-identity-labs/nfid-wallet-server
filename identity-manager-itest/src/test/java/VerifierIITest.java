import org.testng.Assert;
import org.testng.annotations.BeforeClass;
import org.testng.annotations.Test;

import static org.testng.AssertJUnit.assertEquals;


public class VerifierIITest extends BaseDFXITest {

    @BeforeClass
    public void initDfxProject() {
        int i = 0;
        String channel;
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
            call("common/configure_dfx_project", "identity_manager", ROOT_IDENTITY, TTL, TTL_REFRESH, WHITELISTED_PHONE_NUMBERS, DISABLED_HEARTBEAT, BACKUP_CANISTER_ID, verifier_id);
            call("common/configure_verifier", im);

            channel = callDfxCommand("dfx canister call verifier ping'");

            if (++i >= DEFAULT_TRIES)
                System.exit(1);

        } while (channel.isEmpty());
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
        var expected = get("response/data_response", "opt \"+380991111111_SHA2\"", "null", "200");
        Assert.assertEquals(actual, expected);
    }

    @Test(priority = 20)
    public void init_certificate() {
        String init_certificate = "dfx canister call verifier post_certificate '(\"test_domain\")'";
        String key = callDfxCommand(init_certificate);
        key = key.replaceAll(",", "");
        String upgrade = String.format("dfx canister call verifier update_certificate '(%s)'", key);
        String call_certificate = callDfxCommand(upgrade);
        assertEquals("(\n" +
                "  opt record {\n" +
                "    domain = \"test_domain\";\n" +
                "    client_principal = \"sculj-2sjuf-dxqlm-dcv5y-hin5x-zfyvr-tzngf-bt5b5-dwhcc-zbsqf-rae\";\n" +
                "    phone_number_sha2 = opt \"+380991111111_SHA2\";\n" +
                "  },\n" +
                ")\n", call_certificate);
    }

    @Test(priority = 30)
    public void is_owner() {
        String init_certificate = "dfx canister call verifier is_owner '(\"sculj-2sjuf-dxqlm-dcv5y-hin5x-zfyvr-tzngf-bt5b5-dwhcc-zbsqf-rae\")'";
        String key = callDfxCommand(init_certificate);
        assertEquals("(record { data = opt true; error = null; status_code = 200 : nat16 })\n", key);
    }

}
