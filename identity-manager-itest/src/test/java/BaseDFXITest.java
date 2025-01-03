import lombok.SneakyThrows;
import org.apache.commons.io.IOUtils;
import org.testng.annotations.AfterClass;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.Objects;

import static org.testng.AssertJUnit.assertEquals;
import static org.testng.AssertJUnit.assertTrue;

public class BaseDFXITest {
    private final static String PATH = "..";

    static String ROOT_IDENTITY = "";
    final static String TTL = "30";
    final static String TTL_REFRESH = "20";
    final static String WHITELISTED_PHONE_NUMBERS = "null";
    final static String ANCHOR = "1234";
    final static String PHONE = "123456";
    final static String PHONE_SHA2 = "123456_SHA2";
    final static String TOKEN = "1234";
    final static String ANONYMOUS = "anonymous";
    final static String DEFAULT = "default";
    static String BACKUP_CANISTER_ID = "";
    static String HEARTBEAT_PERIOD = "1";
    final static String DISABLED_HEARTBEAT = "1000000";
    final static int DEFAULT_TRIES = 20;


    @AfterClass
    public void stopDfx() {
        call("common/dfx_stop");
        call("common/dfx_stop_2");
    }

    @SneakyThrows
    public void validateWithFormatIdentity(String pathToExpected, String actual) {
        String expected = IOUtils.toString(
                Objects.requireNonNull(this.getClass().getResourceAsStream(pathToExpected)),
                StandardCharsets.UTF_8
        );
        assertEquals(String.format(expected, ROOT_IDENTITY, ROOT_IDENTITY).trim(), actual.trim());
    }

    @SneakyThrows
    public void validateWithFormatIdentity(String pathToExpected, String orPathToExpected, String actual) {
        String expected = IOUtils.toString(
                Objects.requireNonNull(this.getClass().getResourceAsStream(pathToExpected)),
                StandardCharsets.UTF_8
        );
        String expected2 = IOUtils.toString(
                Objects.requireNonNull(this.getClass().getResourceAsStream(orPathToExpected)),
                StandardCharsets.UTF_8
        );
        var formattedExpected1 = String.format(expected, ROOT_IDENTITY, ROOT_IDENTITY).trim();
        var formattedExpected2 = String.format(expected2, ROOT_IDENTITY, ROOT_IDENTITY).trim();
        var formattedActual = actual.trim();
        assertTrue(formattedExpected1.equals(formattedActual) || formattedExpected2.equals(formattedActual));
    }

    @SneakyThrows
    public String get(String file, Object... params) {
        var text = IOUtils.toString(
                Objects.requireNonNull(this.getClass().getResourceAsStream(file)),
                StandardCharsets.UTF_8
        );
        return String.format(text, params).trim().replaceAll("[\\n\\t ]", "");
    }

    @SneakyThrows
    public String call(String path) {
        String dfxCommand = IOUtils.toString(
                Objects.requireNonNull(this.getClass().getResourceAsStream(path)),
                StandardCharsets.UTF_8
        );
        System.out.println(dfxCommand);
        String[] bashScript = new String[]{"/bin/bash", "-c",
                String.format("cd $0 && %s", dfxCommand), getPath(null)};
        return execute(bashScript);
    }

    @SneakyThrows
    public String getScript(String path) {
        return IOUtils.toString(
                Objects.requireNonNull(this.getClass().getResourceAsStream(path)),
                StandardCharsets.UTF_8
        );
    }

    @SneakyThrows
    public String callDfxCommand(String dfxCommand) {
        System.out.printf(dfxCommand.toString());
        String[] bashScript = new String[]{
                "/bin/bash",
                "-c",
                String.format("cd $0 && %s", dfxCommand),
                getPath(null)};
        return execute(bashScript);
    }

    public String call(String file, Object... params) {
        return callDfxCommand(String.format(getScript(file).trim(), params)).trim();
    }

    public String command(String file, Object... params) {
        return callDfxCommand(String.format(getScript(file).trim(), params)).trim().replaceAll("[\\n\\t ]", "");
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
