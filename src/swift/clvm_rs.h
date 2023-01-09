#include <stdint.h>
// cargo lipo --release
void c_sexp_from_stream(const char* hexbytes);
const char* c_curry(const char* program, const char* args);
const char* treehash(const char* program);
const char* int_to_bytes(const long value);
const char* swift_assemble(const char* program);
const char* swift_disassemble(const char* program);
const char* swift_run(const char* program, const char* solution);
const char* swift_first(const char* program);
const char* swift_rest(const char* program);
