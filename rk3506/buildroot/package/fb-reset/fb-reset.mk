################################################################################
# fb-reset
################################################################################

FB_RESET_SITE = /project/device/rootfs
FB_RESET_SITE_METHOD = local

define FB_RESET_BUILD_CMDS
	$(TARGET_CC) $(TARGET_CFLAGS) $(TARGET_LDFLAGS) \
		$(@D)/fb-reset.c -o $(@D)/fb-reset
endef

define FB_RESET_INSTALL_TARGET_CMDS
	$(INSTALL) -D -m 0755 $(@D)/fb-reset $(TARGET_DIR)/usr/bin/fb-reset
endef

$(eval $(generic-package))
