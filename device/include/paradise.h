#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

extern "C" {

int32_t rust_initialize_vad(const void *vad, const uint8_t *driver_path);

} // extern "C"
