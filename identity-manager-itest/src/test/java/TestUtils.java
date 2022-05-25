import org.testng.internal.collections.Pair;

public class TestUtils {

    public static Pair<String, String> cutField(String result, String field) {
        String[] lines = result.split(System.getProperty("line.separator"));
        String fieldValue = "";
        for (int i = 0; i < lines.length; i++) {
            if (lines[i].contains(field)) {
                fieldValue = lines[i];
                lines[i] = "";
            }
        }
        StringBuilder finalStringBuilder = new StringBuilder("");
        for (String s : lines) {
            if (!s.equals("")) {
                finalStringBuilder.append(s).append(System.getProperty("line.separator"));
            }
        }
        return new Pair<String, String>(finalStringBuilder.toString(), fieldValue);
    }
    
}
