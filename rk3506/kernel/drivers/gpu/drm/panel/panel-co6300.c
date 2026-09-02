// SPDX-License-Identifier: GPL-2.0
/*
 * BOE 3.19-inch AMOLED panel using the CO6300 display IC.
 *
 * The command sequence and video timings are based on the P4 ESP-IDF
 * reference driver supplied with the panel module.
 */

#include <linux/delay.h>
#include <linux/gpio/consumer.h>
#include <linux/module.h>
#include <linux/of_device.h>
#include <linux/regulator/consumer.h>

#include <drm/drm_mipi_dsi.h>
#include <drm/drm_modes.h>
#include <drm/drm_panel.h>

#include <video/mipi_display.h>

#define CO6300_DRIVER_VERSION "v0.3-no-native-backlight"

struct co6300 {
	struct drm_panel panel;
	struct mipi_dsi_device *dsi;
	struct regulator *supply;
	struct gpio_desc *reset;
	bool prepared;
};

static inline struct co6300 *to_co6300(struct drm_panel *panel)
{
	return container_of(panel, struct co6300, panel);
}

static int co6300_write(struct co6300 *ctx, u8 cmd, const void *data,
			 size_t len)
{
	struct mipi_dsi_device *dsi = ctx->dsi;
	int ret;

	if (cmd == 0xfe || cmd == 0xf4 || cmd == 0xf5 || cmd == 0x03)
		ret = mipi_dsi_generic_write(dsi, (u8[]){ cmd, *(u8 *)data }, 2);
	else
		ret = mipi_dsi_dcs_write(dsi, cmd, data, len);

	return ret < 0 ? ret : 0;
}

static int co6300_init(struct co6300 *ctx)
{
	static const u8 one_20[] = { 0x20 };
	static const u8 one_5a[] = { 0x5a };
	static const u8 one_59[] = { 0x59 };
	static const u8 one_00[] = { 0x00 };
	static const u8 pixel_format[] = { 0x77 };
	static const u8 madctl[] = { 0x00 };
	static const u8 col_addr[] = { 0x00, 0x00, 0x01, 0x05 };
	static const u8 page_addr[] = { 0x00, 0x00, 0x03, 0x9f };
	static const u8 brightness[] = { 0xff };
	static const u8 hbm_brightness[] = { 0xff };
	static const u8 hbm[] = { 0xff };
	static const u8 normal_brightness[] = { 0x07 };
	int ret;

	/* Unlock and select command page, as documented by the module vendor. */
	ret = co6300_write(ctx, 0xfe, one_20, 1);
	if (ret)
		return ret;
	ret = co6300_write(ctx, 0xf4, one_5a, 1);
	if (ret)
		return ret;
	ret = co6300_write(ctx, 0xf5, one_59, 1);
	if (ret)
		return ret;
	ret = co6300_write(ctx, 0xfe, (u8[]){ 0x80 }, 1);
	if (ret)
		return ret;
	ret = co6300_write(ctx, 0x03, one_00, 1);
	if (ret)
		return ret;
	ret = co6300_write(ctx, 0xfe, one_00, 1);
	if (ret)
		return ret;

	ret = co6300_write(ctx, 0x35, one_00, 1);
	if (ret)
		return ret;
	ret = co6300_write(ctx, 0x3a, pixel_format, 1);
	if (ret)
		return ret;
	ret = co6300_write(ctx, 0x36, madctl, 1);
	if (ret)
		return ret;
	ret = co6300_write(ctx, 0x2a, col_addr, sizeof(col_addr));
	if (ret)
		return ret;
	ret = co6300_write(ctx, 0x2b, page_addr, sizeof(page_addr));
	if (ret)
		return ret;
	ret = co6300_write(ctx, 0x53, (u8[]){ 0x20 }, 1);
	if (ret)
		return ret;
	ret = co6300_write(ctx, 0x51, brightness, 1);
	if (ret)
		return ret;
	ret = co6300_write(ctx, 0x63, hbm_brightness, 1);
	if (ret)
		return ret;

	ret = mipi_dsi_dcs_exit_sleep_mode(ctx->dsi);
	if (ret < 0)
		return ret;
	msleep(60);
	ret = mipi_dsi_dcs_set_display_on(ctx->dsi);
	if (ret < 0)
		return ret;
	ret = co6300_write(ctx, 0x51, hbm, 1);
	if (ret)
		return ret;
	ret = co6300_write(ctx, 0x58, normal_brightness, 1);
	if (ret)
		return ret;
	return 0;
}

static int co6300_prepare(struct drm_panel *panel)
{
	struct co6300 *ctx = to_co6300(panel);
	int ret;

	pr_info("panel-co6300 %s: prepare (native backlight control disabled)\n",
		CO6300_DRIVER_VERSION);

	if (ctx->prepared)
		return 0;

	ret = regulator_enable(ctx->supply);
	if (ret)
		return ret;
	msleep(50);

	if (ctx->reset) {
		gpiod_set_value_cansleep(ctx->reset, 1);
		usleep_range(5000, 6000);
		gpiod_set_value_cansleep(ctx->reset, 0);
		msleep(120);
	}

	ret = co6300_init(ctx);
	if (ret) {
		regulator_disable(ctx->supply);
		return ret;
	}
	ctx->prepared = true;
	return 0;
}

static int co6300_unprepare(struct drm_panel *panel)
{
	struct co6300 *ctx = to_co6300(panel);

	if (!ctx->prepared)
		return 0;
	mipi_dsi_dcs_set_display_off(ctx->dsi);
	mipi_dsi_dcs_enter_sleep_mode(ctx->dsi);
	msleep(120);
	if (ctx->reset)
		gpiod_set_value_cansleep(ctx->reset, 1);
	regulator_disable(ctx->supply);
	ctx->prepared = false;
	return 0;
}

static const struct drm_display_mode co6300_mode = {
	.clock = 16000,
	.hdisplay = 262,
	.hsync_start = 262 + 32,
	.hsync_end = 262 + 32 + 4,
	.htotal = 262 + 32 + 4 + 32,
	.vdisplay = 928,
	.vsync_start = 928 + 8,
	.vsync_end = 928 + 8 + 4,
	.vtotal = 928 + 8 + 4 + 8,
	.width_mm = 42,
	.height_mm = 144,
	.flags = DRM_MODE_FLAG_NHSYNC | DRM_MODE_FLAG_NVSYNC,
};

static int co6300_get_modes(struct drm_panel *panel,
				struct drm_connector *connector)
{
	struct drm_display_mode *mode;

	mode = drm_mode_duplicate(connector->dev, &co6300_mode);
	if (!mode)
		return -ENOMEM;
	drm_mode_set_name(mode);
	drm_mode_probed_add(connector, mode);
	connector->display_info.width_mm = mode->width_mm;
	connector->display_info.height_mm = mode->height_mm;
	return 1;
}

static const struct drm_panel_funcs co6300_panel_funcs = {
	.prepare = co6300_prepare,
	.unprepare = co6300_unprepare,
	.get_modes = co6300_get_modes,
};

static int co6300_probe(struct mipi_dsi_device *dsi)
{
	struct co6300 *ctx;

	pr_info("panel-co6300 %s: probe from tracked driver, native backlight control disabled\n",
		CO6300_DRIVER_VERSION);

	ctx = devm_kzalloc(&dsi->dev, sizeof(*ctx), GFP_KERNEL);
	if (!ctx)
		return -ENOMEM;
	ctx->dsi = dsi;
	ctx->supply = devm_regulator_get(&dsi->dev, "power");
	if (IS_ERR(ctx->supply))
		return dev_err_probe(&dsi->dev, PTR_ERR(ctx->supply),
				     "failed to get power supply\n");
	ctx->reset = devm_gpiod_get_optional(&dsi->dev, "reset", GPIOD_OUT_LOW);
	if (IS_ERR(ctx->reset))
		return dev_err_probe(&dsi->dev, PTR_ERR(ctx->reset),
				     "failed to get reset GPIO\n");

	dsi->lanes = 1;
	dsi->format = MIPI_DSI_FMT_RGB888;
	dsi->mode_flags = MIPI_DSI_MODE_VIDEO | MIPI_DSI_MODE_VIDEO_BURST |
			  MIPI_DSI_MODE_LPM | MIPI_DSI_MODE_NO_EOT_PACKET;

	drm_panel_init(&ctx->panel, &dsi->dev, &co6300_panel_funcs,
		       DRM_MODE_CONNECTOR_DSI);
	drm_panel_of_backlight(&ctx->panel);
	pr_info("panel-co6300 %s: registered without native backlight device\n",
		CO6300_DRIVER_VERSION);
	mipi_dsi_set_drvdata(dsi, ctx);
	drm_panel_add(&ctx->panel);
	return mipi_dsi_attach(dsi);
}

static void co6300_remove(struct mipi_dsi_device *dsi)
{
	struct co6300 *ctx = mipi_dsi_get_drvdata(dsi);

	mipi_dsi_detach(dsi);
	drm_panel_remove(&ctx->panel);
	co6300_unprepare(&ctx->panel);
}

static const struct of_device_id co6300_of_match[] = {
	{ .compatible = "boe,co6300" },
	{ }
};
MODULE_DEVICE_TABLE(of, co6300_of_match);

static struct mipi_dsi_driver co6300_driver = {
	.probe = co6300_probe,
	.remove = co6300_remove,
	.driver = {
		.name = "panel-co6300",
		.of_match_table = co6300_of_match,
	},
};
module_mipi_dsi_driver(co6300_driver);

MODULE_AUTHOR("OpenAI");
MODULE_DESCRIPTION("CO6300 AMOLED panel driver");
MODULE_LICENSE("GPL");
