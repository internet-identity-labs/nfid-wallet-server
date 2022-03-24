import org.testng.annotations.Test;


public class ApplicationITest extends BaseIdentityManagerITest {

    @Test(priority = 10)
    public void createApplicationExpectCorrectResponse() {
        String actual = call("application/req_create_application");
        validateWithFormatIdentity("application/exp_create_application", actual);
    }

    @Test(priority = 20)
    public void createApplicationWithSameNameExpectErrorResponse() {
        String actual = call("application/req_create_application_same_name");
        validateWithFormatIdentity("application/exp_error_unable_to_create", actual);
    }

    @Test(priority = 30)
    public void createApplicationWithSameDomExpectCorrectResponse() {
        String actual = call("application/req_create_application_2");
        validateWithFormatIdentity("application/exp_read_app", actual);
    }

    @Test(priority = 40)
    public void readApplicationsExpectCorrectResponse() {
        String actual = call("application/req_read_applications");
        validateWithFormatIdentity("application/exp_read_app", actual);
    }

    @Test(priority = 50)
    public void isOverLimitExpectCorrectResponse() {
        call("account/req_create_account");
        call("persona/req_create_persona");
        call("persona/req_create_persona_2");
        validateWithFormatIdentity("persona/exp_under_limit_for_app", call("application/req_is_over_limit"));
        call("application/req_create_application_over_limit");
        validateWithFormatIdentity("persona/exp_over_limit_for_app", call("application/req_is_over_limit"));
    }

    @Test(priority = 60)
    public void deleteApplicationIsOverLimitExpectCorrectResponse() {
        validateWithFormatIdentity("application/exp_delete_app", call("application/req_delete_application"));
        validateWithFormatIdentity("persona/exp_under_limit_for_app", call("application/req_is_over_limit"));
    }

}
