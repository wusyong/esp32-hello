CARGO_TARGET_DIR ?= $(realpath ../target)

PROJECT_NAME := app
BUILD_DIR_BASE := $(CARGO_TARGET_DIR)/$(TARGET)/esp-build

include $(IDF_PATH)/make/project.mk

# Use preprocessor macros without an imported `__ctype_ptr__` to prevent
# the “undefined reference to `__locale_ctype_ptr`” linker error.
CFLAGS += -DLWIP_NO_CTYPE_H=1

# Enable the extra fields on the `tm` struct.
CFLAGS += -D__TM_GMTOFF=tm_gmtoff -D__TM_ZONE=tm_zone
