#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

extern "C" {

int32_t rust_io_proc(const void *frame, uint32_t frame_size);

void *rust_new_driver(const char *driver_name, const char *driver_path);

void rust_release_driver(void *driver);

} // extern "C"
