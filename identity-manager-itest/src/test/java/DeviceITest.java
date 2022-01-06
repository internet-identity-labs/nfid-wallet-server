import org.testng.annotations.BeforeClass;
import org.testng.annotations.Test;


public class DeviceITest extends BaseDFXITest {

    @BeforeClass
    public void initAccount() {
        call("account/req_create_account");
    }

    @Test(priority = 1)
    public void createDeviceExpectCorrectResponse() {
        validateWithFormatIdentity("device/exp_create_device", call("device/req_create_device"));
        validateWithFormatIdentity("device/exp_create_device_acc", call("account/req_get_account"));
    }

    @Test(priority = 2)
    public void getDevicesExpectVector() {
        validateWithFormatIdentity("device/exp_read_devices", call("device/req_read_devices"));
    }

    @Test(priority = 3)
    public void getSeveralDevicesExpectVectorSeveral() {
        validateWithFormatIdentity("device/exp_create_device", call("device/req_create_device_2"));
        validateWithFormatIdentity("device/exp_read_devices_2", call("device/req_read_devices"));
    }


}
