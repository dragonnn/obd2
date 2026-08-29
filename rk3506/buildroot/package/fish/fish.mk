################################################################################
#
# fish
#
################################################################################

FISH_VERSION = 3.6.4
FISH_SITE = https://github.com/fish-shell/fish-shell/releases/download/$(FISH_VERSION)
FISH_SOURCE = fish-$(FISH_VERSION).tar.xz
FISH_LICENSE = GPL-2.0
FISH_LICENSE_FILES = COPYING
FISH_CPE_ID_VENDOR = fish-shell
FISH_CPE_ID_PRODUCT = fish
FISH_DEPENDENCIES = ncurses pcre2

FISH_CONF_OPTS = \
	-DWITH_GETTEXT=OFF \
	-DWITH_DOCS=OFF \
	-DBUILD_DOCS=OFF \
	-DFISH_USE_SYSTEM_PCRE2=ON \
	-DCMAKE_INSTALL_SYSCONFDIR=/etc

define FISH_ADD_TO_SHELLS
	grep -qsE '^/usr/bin/fish$$' $(TARGET_DIR)/etc/shells || \
		echo '/usr/bin/fish' >> $(TARGET_DIR)/etc/shells
endef
FISH_TARGET_FINALIZE_HOOKS += FISH_ADD_TO_SHELLS

$(eval $(cmake-package))
