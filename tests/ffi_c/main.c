#include "rpi_imgpatcher.h"
#include <stdio.h>
#include <stdint.h>

int main(int argc,  char **argv) {
  if (argc < 3) {
    printf("usage: %s <path> <content>\n", argv[0]);
    return 1;
  }
  uint32_t err = 0;

  RpiImage* img = rpi_image_new("../fixtures/test.img");
  if (!img) {
    printf("failed to open image\n");
    return 1;
  }

  const char* path = argv[1];
  const char* content = argv[2];

  int64_t written = rpi_image_write_string(
    img,
    path,
    content,
    &err
  );

  if (written < 0) {
    printf("write failed, err=%u\n", err);
    return 1;
  }

  printf("written: %lld bytes\n", written);

  int64_t res = rpi_image_save_to_file(img, "out/out.img");
  if (res != 0) {
    printf("save failed, err=%lld\n", res);
    return 1;
  }

  printf("success\n");
  return 0;
}
