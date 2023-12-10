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
        String accessPoint = call( "device/req_create_access_point", ROOT_IDENTITY);
        Pair<String, String> tuple = TestUtils.cutField(accessPoint, "last_used");
        validateWithFormatIdentity("device/exp_create_access_point", tuple.first());
        Pair<String, String> account = TestUtils.cutField(call("account/req_get_account", "identity_manager"), "last_used");
        validateWithFormatIdentity("device/exp_create_access_point_acc", account.first());
        assertEquals(tuple.second().trim(), account.second().trim());
    }

    @Test(priority = 11)
    public void useAccountExpectNewTimestamp() {
        Pair<String, String> account = TestUtils.cutField(call("account/req_get_account", "identity_manager"), "last_used");
        validateWithFormatIdentity("device/exp_create_access_point_acc", account.first());
        Pair<String, String> lastUsed = TestUtils.cutField(call("device/req_use_access_points"), "last_used");
        assertNotSame(lastUsed.second().trim(), account.second().trim());
    }

    @Test(priority = 20)
    public void getAccessPointExpectVector() {
        Pair<String, String> tuple = TestUtils.cutField(call("device/req_read_access_points"), "last_used");
        validateWithFormatIdentity("device/exp_read_access_points", tuple.first());
    }

    @Test(priority = 30)
    public void tryToCreateSameHashExpectError() {
        validateWithFormatIdentity("device/exp_access_point_exists", call("device/req_create_access_point", ROOT_IDENTITY));
        validateWithFormatIdentity("device/exp_read_access_points", callAndCutLastUsed("device/req_read_access_points"));
    }

    // @Test(priority = 40)
    // public void getSeveralAccessPointsExpectVectorSeveral() {
    //     validateWithFormatIdentity("device/exp_read_access_points_2", callAndCutLastUsed("device/req_create_access_point_2"));
    // }

    @Test(priority = 50)
    public void createExistentAccessPointExpectErrorResponse() {
        validateWithFormatIdentity("device/exp_create_access_point_exists", call("device/req_create_access_point", ROOT_IDENTITY));
    }

    @Test(priority = 60)
    public void updateNotExistentAccessPointExpectErrorResponse() {
        validateWithFormatIdentity("device/exp_create_access_point_not_exists", call("device/req_update_not_existent_access_point"));
    }

    // @Test(priority = 61)
    // public void updateExistentAccessPointExpectVec() {
    //     Pair<String, String> tuple = TestUtils.cutField(call("device/req_update_existent_point", ROOT_IDENTITY), "last_used");
    //     validateWithFormatIdentity("device/exp_update_access_point", tuple.first());
    // }

    // @Test(priority = 70)
    // public void removeExistentAccessPointExpectVector() {
    //     validateWithFormatIdentity("device/exp_remove_access_point", callAndCutLastUsed("device/req_remove_access_point", ROOT_IDENTITY));
    // }

    private String callAndCutLastUsed(String doc, Object... params) {
        Pair<String, String> result = TestUtils.cutField(call(doc, params), "last_used");
        return result.first();
    }

}
