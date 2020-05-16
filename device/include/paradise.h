#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

extern "C" {

int32_t rust_initialize_vad(const void *vad, const char *driver_name, const char *driver_path);

} // extern "C"
