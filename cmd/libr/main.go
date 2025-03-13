package main

import (
	// #include "prog.h"
	"C"
)
import (
	"fmt"
	"runtime/cgo"
	"unsafe"

	"github.com/microsoft/typescript-go/internal/bundled"
	ts "github.com/microsoft/typescript-go/internal/compiler"
	"github.com/microsoft/typescript-go/internal/parser"
	"github.com/microsoft/typescript-go/internal/scanner"
	"github.com/microsoft/typescript-go/internal/vfs/osvfs"

	"github.com/microsoft/typescript-go/internal/ast"
	"github.com/microsoft/typescript-go/internal/core"
	"github.com/microsoft/typescript-go/internal/tspath"
	"github.com/microsoft/typescript-go/internal/vfs"
)


func FfiToGoString(s C.ffi_string) string {
	return C.GoStringN(s.buf, s.length);
}
func FfiToGoStringConsuming(host *FfiCompilerHost, s C.ffi_string) string {
	ret := C.GoStringN(s.buf, s.length);
	C.call_free_ffi_string(&host.inner, s)
	return ret
}

type FfiCompilerHost struct {
	inner C.compiler_host;
	fs vfs.FS;
}

func(host *FfiCompilerHost) DefaultLibraryPath() string {
	return FfiToGoStringConsuming(host, C.call_default_library_path(&host.inner))
}
func(host *FfiCompilerHost) GetCurrentDirectory() string {
	return FfiToGoStringConsuming(host, C.call_get_current_directory(&host.inner))
}
func(host *FfiCompilerHost) NewLine() string {
	return FfiToGoStringConsuming(host, C.call_new_line(&host.inner))
}
func(host *FfiCompilerHost) Trace(msg string) {
	// return
}

func(host *FfiCompilerHost) GetSourceFile(fileName string, path tspath.Path, languageVersion core.ScriptTarget) *ast.SourceFile {
	cFileName := C.CString(fileName)
	cPath := C.CString(string(path))
	cLanguageVersion := int32(languageVersion)
	ffiText := C.call_get_source_file_text(&host.inner, cFileName, cPath, C.int32_t(cLanguageVersion))
	C.free(unsafe.Pointer(cFileName))
	C.free(unsafe.Pointer(cPath))
	text := FfiToGoStringConsuming(host, ffiText)
	// text, _ := h.FS().ReadFile(fileName)
	if tspath.FileExtensionIs(fileName, tspath.ExtensionJson) {
		return parser.ParseJSONText(fileName, path, text)
	}
	return parser.ParseSourceFile(fileName, path, text, languageVersion, scanner.JSDocParsingModeParseForTypeErrors)
}
func(host *FfiCompilerHost) FS() vfs.FS {
	return host.fs;
}

type OpaqueProgram C.uintptr_t

type OpaqueSourceFile C.uintptr_t

//export NewProgram
func NewProgram(options C.program_options) OpaqueProgram {
	opts := ts.ProgramOptions{}
	opts.ConfigFileName = FfiToGoString(options.config_file_name)
	rootFiles := make([]string, options.root_files.length)
	for i := range rootFiles {
		rootFiles[i] = FfiToGoString(C.ffi_string_array_index(&options.root_files, C.int(i)))
	}
	C.call_free_ffi_string_array(&options.host, options.root_files)
	opts.Host = &FfiCompilerHost{inner: options.host, fs: bundled.WrapFS(osvfs.FS())}

	return OpaqueProgram(cgo.NewHandle(ts.NewProgram(opts)))
}

func (s OpaqueSourceFile) cast() *ast.SourceFile {
	if C.uintptr_t(s) == 0 {
		return nil;
	}
	return cgo.Handle(s).Value().(*ast.SourceFile)
}

func (p OpaqueProgram) cast() *ts.Program {
	if C.uintptr_t(p) == 0 {
		return nil;
	}
	return cgo.Handle(p).Value().(*ts.Program)
}

//export CheckSourceFiles
func (p OpaqueProgram) CheckSourceFiles() {
	p.cast().CheckSourceFiles()
}

//export ListFiles
func (p OpaqueProgram) ListFiles() {
	for _, file := range p.cast().SourceFiles() {
		fmt.Println(file.FileName())
	}
}

//export GetSyntacticDiagnostics
func (p OpaqueProgram) GetSyntacticDiagnostics(sourceFile OpaqueSourceFile) C.go_array {
	diagnostics := p.cast().GetSyntacticDiagnostics(sourceFile.cast())
	// converted := make([]C.diagnostic, len(diagnostics))
	converted := C.malloc(C.size_t(len(diagnostics)) * C.sizeof_diagnostic)

	ret := (*[1<<30 - 1]C.diagnostic)(converted)
	for i, d := range diagnostics {
		fileName := d.File().FileName()
		ret[i] = C.diagnostic{message: C.CString(d.Message()), file_name: C.CString(fileName)}
	}
	return C.go_array {
		data: converted,
		len: C.size_t(len(diagnostics)),
	}
}


func main() {}