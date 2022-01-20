import org.testng.annotations.BeforeClass;
import org.testng.annotations.Ignore;
import org.testng.annotations.Test;

import static org.testng.AssertJUnit.assertEquals;


public class PersonaITest extends BaseIdentityManagerITest {

    @BeforeClass
    public void initAccount() {
        call("account/req_create_account");
    }

    @Test(priority = 1)
    public void createPersonaExpectCorrectResponse() {
        validateWithFormatIdentity("persona/exp_create_persona", call("persona/req_create_persona"));
    }

    @Test(priority = 2)
    public void updatePersonasNameExpectCorrectResponse() {
        validateWithFormatIdentity("persona/exp_update_persona", call("persona/req_update_persona"));
    }

    @Test(priority = 3)
    public void addOneMoreExpectList() {
        validateWithFormatIdentity("persona/exp_update_persona_2", call("persona/req_update_persona_2"));
    }

    @Test(priority = 5)
    @Ignore //todo
    public void testPostUpgradePrincipalIndex() {
        call("common/use_default_persona");
        String defaultPrincipal = call("common/get_principal").trim();
        callDfxCommand("cd src && touch test");
        callDfxCommand("dfx canister install identity_manager --mode upgrade --argument '(null)'");
        call("common/use_test_persona");
        String actual = call("account/req_get_account");
        String expected = getScript("persona/exp_update_persona_test_id");
        assertEquals(String.format(expected, defaultPrincipal), actual);
        callDfxCommand("cd src && rm test");
    }

}
