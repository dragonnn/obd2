################################################################################
# obd2-dashboard
################################################################################

OBD2_DASHBOARD_SITE = /project/obd2-dashboard-slint
OBD2_DASHBOARD_SITE_METHOD = local
OBD2_DASHBOARD_LICENSE = MIT
OBD2_DASHBOARD_DEPENDENCIES = eudev fontconfig libdrm libevdev libinput libxkbcommon
OBD2_DASHBOARD_CARGO_ENV = PKG_CONFIG_PATH=$(STAGING_DIR)/usr/lib/pkgconfig:$(STAGING_DIR)/usr/share/pkgconfig
OBD2_DASHBOARD_CARGO_BUILD_OPTS = --no-default-features --features board-kms
OBD2_DASHBOARD_CARGO_INSTALL_OPTS = --no-default-features --features board-kms

define OBD2_DASHBOARD_INSTALL_FONT
	install -d $(TARGET_DIR)/usr/share/fonts/slint
	install -m 0644 /project/obd2-dashboard-slint/vendor/i-slint-common/sharedfontique/Inter-VariableFont.ttf \
		$(TARGET_DIR)/usr/share/fonts/slint/Inter-VariableFont.ttf
endef

OBD2_DASHBOARD_POST_INSTALL_TARGET_HOOKS += OBD2_DASHBOARD_INSTALL_FONT

$(eval $(cargo-package))
