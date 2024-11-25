// __SOURCE__
import java.io.File;
import java.io.FileOutputStream;
import java.util.Base64;
import java.util.zip.GZIPInputStream;
import kotlin.system.exitProcess

const val payload = "__PAYLOAD__"

fun main(args: Array<String>) {
    val file = File.createTempFile("binary", null)

    Base64
        .getDecoder()
        .decode(payload)
        .inputStream()
        .let { GZIPInputStream(it) }
        .use { input -> FileOutputStream(file).use { input.copyTo(it) } }

    assert(file.setExecutable(true))

    ProcessBuilder(file.absolutePath, *args)
        .inheritIO()
        .start()
        .waitFor()
        .let { exitProcess(it) }
}
