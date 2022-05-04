import org.testng.annotations.BeforeClass;
import org.testng.annotations.Test;
import org.testng.internal.collections.Pair;

import static org.testng.AssertJUnit.assertEquals;
import static org.testng.AssertJUnit.assertNotSame;


public class AccessPointITest extends BaseIdentityManagerITest {

    @BeforeClass
    public void initAccount() {
        call("account/req_create_account");
    }

    @Test(priority = 10)
    public void createAccessPointExpectCorrectResponse() {
        String accessPoint = call("device/req_create_access_point");
        Pair<String, String> tuple = cutField(accessPoint, "last_used");
        validateWithFormatIdentity("device/exp_create_access_point", tuple.first());
        Pair<String, String> account = cutField(call("account/req_get_account", "identity_manager"), "last_used");
        validateWithFormatIdentity("device/exp_create_access_point_acc", account.first());
        assertEquals(tuple.second().trim(), account.second().trim());
    }

    @Test(priority = 11)
    public void useAccountExpectNewTimestamp() {
        Pair<String, String> account = cutField(call("account/req_get_account", "identity_manager"), "last_used");
        validateWithFormatIdentity("device/exp_create_access_point_acc", account.first());
        Pair<String, String> lastUsed = cutField(call("device/req_use_access_points"), "last_used");
        assertNotSame(lastUsed.second().trim(), account.second().trim());
    }

    @Test(priority = 20)
    public void getAccessPointExpectVector() {
        Pair<String, String> tuple = cutField(call("device/req_read_access_points"), "last_used");
        validateWithFormatIdentity("device/exp_read_access_points", tuple.first());
    }

    @Test(priority = 30)
    public void tryToCreateSameHashExpectError() {
        validateWithFormatIdentity("device/exp_access_point_exists", call("device/req_create_access_point"));
        validateWithFormatIdentity("device/exp_read_access_points", callAndCutLastUsed("device/req_read_access_points"));
    }

    @Test(priority = 40)
    public void getSeveralAccessPointsExpectVectorSeveral() {
        validateWithFormatIdentity("device/exp_read_access_points_2", callAndCutLastUsed("device/req_create_access_point_2"));
    }

    @Test(priority = 50)
    public void createExistentAccessPointExpectErrorResponse() {
        validateWithFormatIdentity("device/exp_create_access_point_exists", call("device/req_create_access_point"));
    }

    @Test(priority = 70)
    public void removeExistentAccessPointExpectVector() {
        validateWithFormatIdentity("device/exp_remove_access_point", callAndCutLastUsed("device/req_remove_access_point"));
    }

    private String callAndCutLastUsed(String doc) {
        Pair<String, String> result = cutField(call(doc), "last_used");
        return result.first();
    }


    private Pair<String, String> cutField(String result, String field) {
        String[] lines = result.split(System.getProperty("line.separator"));
        String fieldValue = "";
        for (int i = 0; i < lines.length; i++) {
            if (lines[i].contains(field)) {
                fieldValue = lines[i];
                lines[i] = "";
            }
        }
        StringBuilder finalStringBuilder = new StringBuilder("");
        for (String s : lines) {
            if (!s.equals("")) {
                finalStringBuilder.append(s).append(System.getProperty("line.separator"));
            }
        }
        return new Pair(finalStringBuilder.toString(), fieldValue);
    }

}
