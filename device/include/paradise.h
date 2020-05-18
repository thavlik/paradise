#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

extern "C" {

void rust_io_proc(void *driver);

void *rust_new_driver(const char *driver_name, const char *driver_path);

void rust_release_driver(void *driver);

} // extern "C"
