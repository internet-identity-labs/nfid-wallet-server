import org.testng.annotations.BeforeClass;
import org.testng.annotations.Test;

import static org.testng.AssertJUnit.assertEquals;


public class PersonaITest extends BaseIdentityManagerITest {

    @BeforeClass
    public void initAccount() {
        call("account/req_create_account");
    }

    @Test(priority = 21)
    public void readPersonasExpectList() {
        validateWithFormatIdentity("persona/exp_list_personas", call("persona/req_read_personas"));
    }

}
