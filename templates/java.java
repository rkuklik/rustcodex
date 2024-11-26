// __SOURCE__
import java.io.ByteArrayInputStream;
import java.io.File;
import java.io.FileOutputStream;
import java.util.ArrayList;
import java.util.Base64;
import java.util.List;
import java.util.zip.GZIPInputStream;

public class Main {
    private static final String payload = "__PAYLOAD__";

    public static void main(String[] args) throws Exception {
        File file = File.createTempFile("binary", null);

        try (
                ByteArrayInputStream binary = new ByteArrayInputStream(Base64.getDecoder().decode(payload));
                GZIPInputStream input = new GZIPInputStream(binary);
                FileOutputStream output = new FileOutputStream(file)
        ) {
            byte[] buf = new byte[8 * 1024];
            while (true) {
                int read = input.read(buf, 0, buf.length);
                if (read == -1) {
                    break;
                }
                output.write(buf, 0, read);
            }
        }

        file.setExecutable(true);

        List<String> cmd = new ArrayList<String>(args.length + 1);
        cmd.add(file.getAbsolutePath());
        for (String arg : args) {
            cmd.add(arg);
        }
        int code = new ProcessBuilder(cmd).inheritIO().start().waitFor();
        System.exit(code);
    }
}
