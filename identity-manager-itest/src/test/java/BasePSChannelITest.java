import org.testng.annotations.BeforeClass;

public class BasePSChannelITest extends BaseDFXITest {

    @BeforeClass
    public void initDfxProject() {
        int i = 0;
        String channel;
        do {
            call("common/dfx_stop");
            callDfxCommand("rm -rf .dfx");
            call("common/use_default_persona");
            ROOT_IDENTITY = call("common/get_principal").trim();
            call("common/init_dfx_project");
            var command = String.format(getScript("common/deploy_dfx_project").trim(), ROOT_IDENTITY);
            callDfxCommand(command);
            channel = call("common/req_create_topic");

            if (++i >= DEFAULT_TRIES)
                System.exit(1);

        } while (channel.isEmpty());
    }

}
