import org.testng.annotations.BeforeClass;
import org.testng.annotations.Test;

import static org.testng.AssertJUnit.assertEquals;


public class PersonaITest extends BaseIdentityManagerITest {

    @BeforeClass
    public void initAccount() {
        call("account/req_create_account");
    }

    @Test(priority = 10)
    public void createNfidPersonaExpectCorrectResponse() {
        validateWithFormatIdentity("persona/exp_nfid_persona", call("persona/req_create_nfid_persona"));
    }

    @Test(priority = 20)
    public void createIIPersonaExpectCorrectResponse() {
        validateWithFormatIdentity("persona/exp_ii_persona", call("persona/req_create_ii_persona"));
    }

    @Test(priority = 30)
    public void addOneMoreExpectList() {
        validateWithFormatIdentity("persona/exp_nfid_persona_2", call("persona/req_create_nfid2_persona"));
    }

    @Test(priority = 40)
    public void addInvalidIIOneMoreExpectList() {
        validateWithFormatIdentity("persona/exp_invalid_persona", call("persona/req_create_invalid_ii_persona"));
    }

    @Test(priority = 50)
    public void addInvalidIIAnchExpectError() {
        validateWithFormatIdentity("persona/exp_invalid_anch", call("persona/req_create_ii_persona_anch_exists"));
    }

    @Test(priority = 60)
    public void addInvalidIIAnchRootExpectError() {
        validateWithFormatIdentity("persona/exp_invalid_anch", call("persona/req_create_ii_persona_anch_exists_root"));
    }

    @Test(priority = 70)
    public void addOverDomainLimitExpectError() {
        call("persona/req_create_application");
        validateWithFormatIdentity("persona/exp_over_limit_for_app_bool", call("application/req_is_over_limit"));
        validateWithFormatIdentity("persona/exp_under_limit_for_app", call("persona/req_create_ii_persona_over_limit_domain"));
    }

    @Test(priority = 80)
    public void testPostUpgradePrincipalIndex() {
        callDfxCommand("cd src && touch test");
        callDfxCommand("dfx build");
        callDfxCommand("dfx canister install --all --mode upgrade");
        validateWithFormatIdentity("persona/exp_list_personas", call("persona/req_read_personas"));
        callDfxCommand("cd src && rm test");
    }

}
