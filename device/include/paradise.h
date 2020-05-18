#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

extern "C" {

void *rust_initialize_vad(const char *driver_name, const char *driver_path);

int32_t rust_io_proc(const void *frame, uint32_t frame_size);

} // extern "C"
