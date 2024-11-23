// __SOURCE__
package main

import (
	"compress/gzip"
	"encoding/base64"
	"fmt"
	"io"
	"os"
	"strings"
	"syscall"
)

var payload = "__PAYLOAD__"

func main() {
	output := try(os.CreateTemp("/tmp", ""))
	expect(output.Chmod(0700))
	name := output.Name()

	input := try(gzip.NewReader(base64.NewDecoder(base64.StdEncoding, strings.NewReader(payload))))
	try(io.Copy(output, input))
	expect(input.Close())
	expect(output.Close())

	argv := os.Args
	argv[0] = "binary"
	expect(syscall.Exec(name, argv, os.Environ()))
}

func try[T any](output T, err error) T {
	expect(err)
	return output
}

func expect(err error) {
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}
