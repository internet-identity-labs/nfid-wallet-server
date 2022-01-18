import org.testng.annotations.BeforeClass;

public class BaseIdentityManagerITest extends BaseDFXITest {
    private final static int DEFAULT_TRIES = 20;

    @BeforeClass
    public void initDfxProject() {
        int i = 0;
        String identity_manager;
        do {
            call("common/dfx_stop");
            callDfxCommand("rm -rf .dfx");
            call("common/use_default_persona");
            ROOT_IDENTITY = call("common/get_principal").trim();
            call("common/init_dfx_project");
            call("common/deploy_dfx_project");
            var command = String.format(getScript("common/configure_dfx_project").trim(), ROOT_IDENTITY);
            identity_manager = callDfxCommand(command);

            if (++i >= DEFAULT_TRIES)
                System.exit(1);

        } while (identity_manager.isEmpty());
        call("token/req_post_token_default");
    }

}
