// __SOURCE__
using System;
using System.Diagnostics;
using System.IO;
using System.IO.Compression;
using System.Text;

class RustCoDex {
    private static string payload = "__PAYLOAD__";

    static void Main(string[] args) {
        string path = Path.GetTempFileName();

        using (var memory = new MemoryStream(Convert.FromBase64String(payload)))
        using (var input = new GZipStream(memory, CompressionMode.Decompress))
        using (var output = File.OpenWrite(path)) {
            byte[] buf = new byte[8 * 1024];
            while (true) {
                int read = input.Read(buf, 0, buf.Length);
                if (read == 0) {
                    break;
                }
                output.Write(buf, 0, read);
            }
        }

        #pragma warning disable CA1416
        UnixFileMode mode = UnixFileMode.UserRead | UnixFileMode.UserWrite | UnixFileMode.UserExecute;
        File.SetUnixFileMode(path, mode);
        #pragma warning restore CA1416
        
        var info = new ProcessStartInfo {
            FileName = path,
            RedirectStandardError = false,
            RedirectStandardInput = false,
            RedirectStandardOutput = false,
            UseShellExecute = false,
            CreateNoWindow = true,
        };
        foreach (string arg in args) {
            info.ArgumentList.Add(arg);
        }
        var process = new Process();
        process.StartInfo = info;
        process.Start();
        process.WaitForExit();
        Environment.Exit(process.ExitCode);
    }
}
