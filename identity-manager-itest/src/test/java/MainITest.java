import org.testng.annotations.Test;

public class MainITest extends BaseIdentityManagerITest {

    @Test(priority = 1)
    public void configureWhenTheUserExpectOk() {
        var user = call("common/get_principal").trim();
        var command = String.format(getScript("common/configure_dfx_project").trim(), user);
        String actual = callDfxCommand(command);
        validateWithFormatIdentity("main/exp_ok", actual);
    }

    @Test(priority = 2)
    public void configureWhenOtherUserExpectFail() {
        call("common/use_test_persona");
        var user = call("common/get_principal").trim();
        var command = String.format(getScript("common/configure_dfx_project").trim(), user);
        String actual = callDfxCommand(command);
        validateWithFormatIdentity("main/exp_fail", actual);
    }

}
