#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

extern "C" {

void rust_io_proc(void *driver, const uint8_t *buffer, uint32_t buffer_size, double sample_time);

void *rust_new_driver(const char *driver_name, const char *driver_path);

void rust_stop_driver(void *driver);

} // extern "C"
