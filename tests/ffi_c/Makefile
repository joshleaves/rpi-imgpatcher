CC = clang
CFLAGS = -I../../include -Wall -Wextra -O2
UNAME_S := $(shell uname -s)

ifeq ($(UNAME_S),Darwin)
  LDFLAGS = ../../target/release/librpi_imgpatcher.a -framework CoreFoundation
else
  LDFLAGS = ../../target/release/librpi_imgpatcher.a
endif

TARGET = out/test
SRC = main.c

.PHONY: all run clean

all: $(TARGET)

$(TARGET): $(SRC)
	$(CC) $(CFLAGS) $(SRC) $(LDFLAGS) -o $(TARGET)

run: $(TARGET)
	./$(TARGET) $(ARGS)

clean:
	rm -f $(TARGET)
	rm -rf out/*.img
	rm -rf out/*.img.xz
