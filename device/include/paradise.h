#ifndef PARADISE_H
#define PARADISE_H

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

struct DriverHandle {
  const void *strong;
  const void *weak;
};

extern "C" {

void rust_io_proc(const void *driver,
                  const uint8_t *buffer,
                  uint32_t buffer_size,
                  double sample_time);

DriverHandle rust_new_driver(const char *driver_name, const char *driver_path);

void rust_stop_driver(const void *driver);

} // extern "C"

#endif // PARADISE_H
