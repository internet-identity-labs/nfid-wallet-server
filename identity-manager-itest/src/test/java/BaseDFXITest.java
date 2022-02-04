import lombok.SneakyThrows;
import org.apache.commons.io.IOUtils;
import org.testng.annotations.AfterClass;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;

import static org.testng.AssertJUnit.assertEquals;

public class BaseDFXITest {
    private final static String PATH = "..";

    static String ROOT_IDENTITY = "";

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
        assertEquals(String.format(expected, ROOT_IDENTITY).trim(), actual.trim());
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

    public String call(String dfxCommand, Object... params) {
        return callDfxCommand(String.format(getScript(dfxCommand).trim(), params));
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
