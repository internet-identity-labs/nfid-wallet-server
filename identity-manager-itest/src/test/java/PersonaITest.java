import org.testng.annotations.BeforeClass;
import org.testng.annotations.Test;

import static org.testng.AssertJUnit.assertEquals;


public class PersonaITest extends BaseIdentityManagerITest {

    @BeforeClass
    public void initAccount() {
        call("account/req_create_account");
    }

    @Test(priority = 1)
    public void createNfidPersonaExpectCorrectResponse() {
        validateWithFormatIdentity("persona/exp_nfid_persona", call("persona/req_create_nfid_persona"));
    }

    @Test(priority = 2)
    public void createIIPersonaExpectCorrectResponse() {
        validateWithFormatIdentity("persona/exp_ii_persona", call("persona/req_create_ii_persona"));
    }

    @Test(priority = 3)
    public void addOneMoreExpectList() {
        validateWithFormatIdentity("persona/exp_nfid_persona_2", call("persona/req_create_nfid2_persona"));
    }

    @Test(priority = 4)
    public void testPostUpgradePrincipalIndex() {
        callDfxCommand("cd src && touch test");
        callDfxCommand("dfx build");
        callDfxCommand("dfx canister install --all --mode upgrade");
        validateWithFormatIdentity("persona/exp_list_personas", call("persona/req_read_personas"));
        callDfxCommand("cd src && rm test");
    }

}
