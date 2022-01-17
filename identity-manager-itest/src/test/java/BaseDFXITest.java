import lombok.SneakyThrows;
import org.apache.commons.io.IOUtils;
import org.testng.annotations.AfterClass;
import org.testng.annotations.BeforeClass;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;

import static org.testng.AssertJUnit.assertEquals;

public class BaseDFXITest {
    private final static String PATH = "..";
    private final static int DEFAULT_TRIES = 20;

    static String ROOT_IDENTITY = "";

    @BeforeClass
    public void initDfxProject() {
        int i = 0;
        String actual;
        do {
            call("common/dfx_stop");
            callDfxCommand("rm -rf .dfx");
            call("common/use_default_persona");
            ROOT_IDENTITY = call("common/get_principal").trim();
            call("common/init_dfx_project");
            call("common/deploy_dfx_project");
            var command = String.format(getScript("common/configure_dfx_project").trim(), ROOT_IDENTITY);
            actual = callDfxCommand(command);

            if (++i >= DEFAULT_TRIES)
                System.exit(1);

        } while (actual.isEmpty());
        call("token/req_post_token_default");
    }

    @AfterClass
    public void stopDfx() {
        call("common/dfx_stop");
    }

    @SneakyThrows
    public void validateWithFormatIdentity(String pathToExpected, String actual) {
        String expected = IOUtils.toString(
                this.getClass().getResourceAsStream(pathToExpected),
                StandardCharsets.UTF_8
        );
        assertEquals(String.format(expected, ROOT_IDENTITY), actual);
    }

    @SneakyThrows
    public String call(String path) {
        String dfxCommand = IOUtils.toString(
                this.getClass().getResourceAsStream(path),
                StandardCharsets.UTF_8
        );
        String[] bashScript = new String[]{"/bin/bash", "-c",
                String.format("cd $0 && %s", dfxCommand), getPath(null)};
        return execute(bashScript);
    }

    @SneakyThrows
    public String getScript(String path) {
        return IOUtils.toString(
                this.getClass().getResourceAsStream(path),
                StandardCharsets.UTF_8
        );
    }

    @SneakyThrows
    public String callDfxCommand(String dfxCommand) {
        String[] bashScript = new String[]{"/bin/bash", "-c",
                String.format("cd $0 && %s", dfxCommand), getPath(null)};
        return execute(bashScript);
    }

    public String getPath(String somePath) {
        return PATH;
    }

    private String execute(String[] bashCommand) throws IOException, InterruptedException {
        String line;
        String result = "";
        ProcessBuilder pb = new ProcessBuilder(bashCommand);
        Process pr = Runtime.getRuntime().exec(bashCommand);
        pb.command(bashCommand);
        pr.waitFor();
        BufferedReader reader2 =
                new BufferedReader(new InputStreamReader(pr.getInputStream()));
        while ((line = reader2.readLine()) != null) {
            System.out.print(line + "\n");
            result += line + "\n";
        }
        return result;
    }

}
