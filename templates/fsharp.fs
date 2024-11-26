// __SOURCE__
open System
open System.Diagnostics
open System.IO
open System.IO.Compression

module RustCoDex =
    let payload = "__PAYLOAD__"

    [<EntryPoint>]
    let main (args: string[]) =
        let path = Path.GetTempFileName()

        do
            use memory = new MemoryStream(Convert.FromBase64String payload)
            use input = new GZipStream(memory, CompressionMode.Decompress)
            use output = File.OpenWrite path
            let buf = Array.create (8 * 1024) 0uy
            let input () = input.Read(buf, 0, buf.Length)
            let mutable read = input ()

            while read <> 0 do
                output.Write(buf, 0, read)
                read <- input ()

        if not (OperatingSystem.IsWindows()) then
            let mode =
                UnixFileMode.UserRead ||| UnixFileMode.UserWrite ||| UnixFileMode.UserExecute

            File.SetUnixFileMode(path, mode)

        let info =
            ProcessStartInfo(
                FileName = path,
                RedirectStandardError = false,
                RedirectStandardInput = false,
                RedirectStandardOutput = false,
                UseShellExecute = false,
                CreateNoWindow = true
            )

        for arg in args do
            info.ArgumentList.Add arg

        let program = new Process()
        program.StartInfo <- info
        program.Start() |> ignore
        program.WaitForExit()
        program.ExitCode
