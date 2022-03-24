import org.testng.annotations.BeforeClass;
import org.testng.annotations.Test;


public class AccessPointITest extends BaseIdentityManagerITest {

    @BeforeClass
    public void initAccount() {
        call("account/req_create_account");
    }

    @Test(priority = 1)
    public void createAccessPointExpectCorrectResponse() {
        validateWithFormatIdentity("device/exp_create_access_point", call("device/req_create_access_point"));
        validateWithFormatIdentity("device/exp_create_access_point_acc", call("account/req_get_account", "identity_manager"));
    }

    @Test(priority = 2)
    public void getAccessPointExpectVector() {
        validateWithFormatIdentity("device/exp_read_access_points", call("device/req_read_access_points"));
    }

    @Test(priority = 3)
    public void tryToCreateSameHashExpectError() {
        validateWithFormatIdentity("device/exp_access_point_exists", call("device/req_create_access_point_exist"));
        validateWithFormatIdentity("device/exp_read_access_points", call("device/req_read_access_points"));
    }

    @Test(priority = 4)
    public void getSeveralAccessPointsExpectVectorSeveral() {
        validateWithFormatIdentity("device/exp_read_access_points_2", call("device/req_create_access_point_2"));
    }

    @Test(priority = 5)
    public void createExistentAccessPointExpectErrorResponse() {
        validateWithFormatIdentity("device/exp_create_access_point_exists", call("device/req_create_access_point"));
    }

    @Test(priority = 7)
    public void removeExistentAccessPointExpectVector() {
        validateWithFormatIdentity("device/exp_remove_access_point", call("device/req_remove_access_point"));
    }


}
