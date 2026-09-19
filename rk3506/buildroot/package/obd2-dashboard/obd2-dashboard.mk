################################################################################
# obd2-dashboard
################################################################################

OBD2_DASHBOARD_SITE = /project/obd2-dashboard-slint
OBD2_DASHBOARD_SITE_METHOD = local
# Preserve unchanged mtimes and Cargo output during Buildroot's initial sync.
OBD2_DASHBOARD_OVERRIDE_SRCDIR_RSYNC_EXCLUSIONS = \
	--checksum --no-times --exclude /target --exclude '/.stamp*'

# Buildroot uses rsync -u, which skips edits with older source timestamps.
# Follow it with a checksum sync without -u, also removing deleted sources.
# Changed files get fresh mtimes so Cargo notices them; cache/stamps survive.
define OBD2_DASHBOARD_SYNC_SOURCES
	rsync -a --delete --chmod=u=rwX,go=rX \
		$(OBD2_DASHBOARD_OVERRIDE_SRCDIR_RSYNC_EXCLUSIONS) \
		$(RSYNC_VCS_EXCLUSIONS) $(OBD2_DASHBOARD_SITE)/ $(@D)/
endef
OBD2_DASHBOARD_POST_RSYNC_HOOKS += OBD2_DASHBOARD_SYNC_SOURCES
OBD2_DASHBOARD_LICENSE = MIT
OBD2_DASHBOARD_DEPENDENCIES = eudev fontconfig libdrm libevdev libinput libxkbcommon rockchip-rga
OBD2_DASHBOARD_CARGO_ENV = PKG_CONFIG_PATH=$(STAGING_DIR)/usr/lib/pkgconfig:$(STAGING_DIR)/usr/share/pkgconfig
OBD2_DASHBOARD_CARGO_BUILD_OPTS = --no-default-features --features board-kms
OBD2_DASHBOARD_CARGO_INSTALL_OPTS = --no-default-features --features board-kms
OBD2_DASHBOARD_SLINT_FONT = $(firstword $(wildcard /project/obd2-dashboard-slint/vendor/i-slint-common*/sharedfontique/Inter-VariableFont.ttf))

define OBD2_DASHBOARD_INSTALL_FONT
	install -d $(TARGET_DIR)/usr/share/fonts/slint
	install -m 0644 $(OBD2_DASHBOARD_SLINT_FONT) \
		$(TARGET_DIR)/usr/share/fonts/slint/Inter-VariableFont.ttf
endef

OBD2_DASHBOARD_POST_INSTALL_TARGET_HOOKS += OBD2_DASHBOARD_INSTALL_FONT

$(eval $(cargo-package))
