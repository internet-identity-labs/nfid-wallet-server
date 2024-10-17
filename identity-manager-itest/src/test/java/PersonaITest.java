import org.testng.annotations.BeforeClass;
import org.testng.annotations.Test;

import static org.testng.AssertJUnit.assertEquals;


public class PersonaITest extends BaseIdentityManagerITest {

    @BeforeClass
    public void initAccount() {
        call("account/req_create_account");
    }

    @Test(priority = 10)
    public void createPersonaExpectCorrectResponse() {
        validateWithFormatIdentity("persona/exp_persona", call("persona/req_create_persona"));
    }

    @Test(priority = 20)
    public void addOneMoreExpectList() {
        validateWithFormatIdentity("persona/exp_create_persona_2", call("persona/req_create_persona_2"));
    }

    @Test(priority = 21)
    public void readPersonasExpectList() {
        validateWithFormatIdentity("persona/exp_list_personas", call("persona/req_read_personas"));
    }

    @Test(priority = 30)
    public void addInvalidIIOneMoreExpectList() {
        validateWithFormatIdentity("persona/exp_invalid_persona", call("persona/req_create_invalid_persona"));
    }

    @Test(priority = 50)
    public void updatePersona() {
        validateWithFormatIdentity("persona/exp_update_persona", call("persona/req_update_persona"));
    }

    @Test(priority = 51)
    public void updateUnExistentPersona() {
        validateWithFormatIdentity("persona/exp_incorrect_update_persona", call("persona/req_update_incorrect_persona"));
    }

}
