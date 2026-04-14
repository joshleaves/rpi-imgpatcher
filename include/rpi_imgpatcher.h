#ifndef RPI_IMGPATCHER_H
#define RPI_IMGPATCHER_H

#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct RpiImage RpiImage;

struct RpiImage *rpi_image_new(const char *image_path);

int64_t rpi_image_write_file(struct RpiImage *rpi_image,
                             const char *fat_path,
                             const char *file,
                             uint32_t *out_error);

int64_t rpi_image_write_string(struct RpiImage *rpi_image,
                               const char *fat_path,
                               const char *string,
                               uint32_t *out_error);

int64_t rpi_image_write_bytes(struct RpiImage *rpi_image,
                              const char *fat_path,
                              const uint8_t *bytes_ptr,
                              uintptr_t bytes_len,
                              uint32_t *out_error);

int64_t rpi_image_append_string(struct RpiImage *rpi_image,
                                const char *fat_path,
                                const char *string,
                                uint32_t *out_error);

int64_t rpi_image_append_bytes(struct RpiImage *rpi_image,
                               const char *fat_path,
                               const uint8_t *bytes_ptr,
                               uintptr_t bytes_len,
                               uint32_t *out_error);

int64_t rpi_image_save_to_file(struct RpiImage *rpi_image, const char *file);

char * rpi_imgpatcher_last_error_message();

#endif  /* RPI_IMGPATCHER_H */
