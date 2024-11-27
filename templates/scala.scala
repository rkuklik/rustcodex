// __SOURCE__
import java.io.ByteArrayInputStream
import java.io.File
import java.io.FileOutputStream
import java.util.ArrayList
import java.util.Base64
import java.util.List
import java.util.zip.GZIPInputStream

import scala.util.Using

object RustCoDex:
  private val payload = "__PAYLOAD__"

  def main(args: Array[String]): Unit =
    val file = File.createTempFile("binary", null);

    Using.Manager { use =>
      val binary = use(ByteArrayInputStream(Base64.getDecoder().decode(payload)))
      val input = use(GZIPInputStream(binary))
      val output = use(FileOutputStream(file))
      val buf = Array.ofDim[Byte](8 * 1024)
      val input = () => input.read(buf, 0, buf.length)
      var read = input()
      while read != -1
      do
        output.write(buf, 0, read)
        read = input()
    }

    file.setExecutable(true)

    val cmd = ArrayList[String](args.length + 1)
    cmd.add(file.getAbsolutePath())
    for (arg <- args) {
      cmd.add(arg)
    }
    val code = ProcessBuilder(cmd)
      .inheritIO()
      .start()
      .waitFor()
    sys.exit(code)
