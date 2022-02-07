import org.testng.annotations.Test;

public class MainITest extends BaseIdentityManagerITest {

    @Test(priority = 1)
    public void configureWhenTheUserExpectOk() {
        var user = call("common/get_principal").trim();
        String actual = call("common/configure_dfx_project", KEY, user, TTL, TTL_REFRESH, WHITELISTED_PHONE_NUMBERS);
        validateWithFormatIdentity("main/exp_ok", actual);
    }

    @Test(priority = 2)
    public void configureWhenOtherUserExpectFail() {
        call("common/use_test_persona");
        var user = call("common/get_principal").trim();
        String actual = call("common/configure_dfx_project", KEY, user, TTL, TTL_REFRESH, WHITELISTED_PHONE_NUMBERS);
        validateWithFormatIdentity("main/exp_fail", actual);
    }

}
