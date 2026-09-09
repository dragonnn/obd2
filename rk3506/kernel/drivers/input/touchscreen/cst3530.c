// SPDX-License-Identifier: GPL-2.0-only
/*
 * Hynitron CST3530, as fitted to the AM319M262928ZS AMOLED module.
 * Wire protocol: vendor esp_lcd_touch_cst3530 ESP-IDF sample in amoled/.
 * No chip-ID, sleep command, or hardware contact-ID format is documented.
 */

#include <linux/delay.h>
#include <linux/gpio/consumer.h>
#include <linux/i2c.h>
#include <linux/input.h>
#include <linux/input/mt.h>
#include <linux/input/touchscreen.h>
#include <linux/interrupt.h>
#include <linux/module.h>
#include <linux/mutex.h>
#include <linux/of.h>
#include <linux/pm.h>
#include <linux/regulator/consumer.h>

#include "cst3530-report.h"

struct cst3530 {
	struct i2c_client *client;
	struct input_dev *input;
	struct gpio_desc *reset;
	struct touchscreen_properties prop;
	/* Serializes report processing with system suspend and resume. */
	struct mutex lock;
	bool suspended;
};

static int cst3530_ack(struct cst3530 *ts)
{
	const u8 cmd[] = { 0xd0, 0x00, 0x02, 0xab };
	int ret;

	ret = i2c_master_send(ts->client, cmd, sizeof(cmd));
	usleep_range(100, 200);
	return ret == sizeof(cmd) ? 0 : ret < 0 ? ret : -EIO;
}

static int cst3530_read(struct cst3530 *ts, u8 *buf)
{
	u8 cmd[] = { 0xd0, 0x07, 0x00, 0x00 };
	struct i2c_msg msgs[] = {
		{
			.addr = ts->client->addr,
			.len = sizeof(cmd),
			.buf = cmd,
		}, {
			.addr = ts->client->addr,
			.flags = I2C_M_RD,
			.len = CST3530_REPORT_SIZE,
			.buf = buf,
		},
	};
	int ret;

	/* Combined transaction preserves the sample's repeated START. */
	ret = i2c_transfer(ts->client->adapter, msgs, ARRAY_SIZE(msgs));
	return ret == ARRAY_SIZE(msgs) ? 0 : ret < 0 ? ret : -EIO;
}

static void cst3530_release(struct cst3530 *ts)
{
	input_mt_sync_frame(ts->input);
	input_sync(ts->input);
}

static void cst3530_update(struct cst3530 *ts)
{
	struct cst3530_contact contacts[CST3530_MAX_CONTACTS];
	struct input_mt_pos pos[CST3530_MAX_CONTACTS];
	int slots[CST3530_MAX_CONTACTS];
	u8 buf[CST3530_REPORT_SIZE];
	int ret, ack_ret, count, i;

	mutex_lock(&ts->lock);
	if (ts->suspended)
		goto unlock;

	ret = cst3530_read(ts, buf);
	/* Also acknowledge malformed/failed reads to avoid wedging INT low. */
	ack_ret = cst3530_ack(ts);
	if (!ret)
		ret = ack_ret;
	if (ret)
		goto error;

	count = cst3530_decode_report(buf, contacts, ts->prop.max_x,
				      ts->prop.max_y);
	if (count < 0) {
		ret = count;
		goto error;
	}

	for (i = 0; i < count; i++)
		touchscreen_set_mt_pos(&pos[i], &ts->prop,
				       contacts[i].x, contacts[i].y);

	/* The sample does not expose contact IDs; track by position. */
	ret = input_mt_assign_slots(ts->input, slots, pos, count, 0);
	if (ret)
		goto error;

	for (i = 0; i < count; i++) {
		input_mt_slot(ts->input, slots[i]);
		input_mt_report_slot_state(ts->input, MT_TOOL_FINGER, true);
		input_report_abs(ts->input, ABS_MT_POSITION_X, pos[i].x);
		input_report_abs(ts->input, ABS_MT_POSITION_Y, pos[i].y);
		input_report_abs(ts->input, ABS_MT_PRESSURE, contacts[i].pressure);
	}
	cst3530_release(ts);
	goto unlock;

error:
	dev_err_ratelimited(&ts->client->dev, "Touch report failed: %d\n", ret);
	/* A bad frame must not leave a finger permanently pressed. */
	cst3530_release(ts);
unlock:
	mutex_unlock(&ts->lock);
}

static irqreturn_t cst3530_irq(int irq, void *data)
{
	cst3530_update(data);
	return IRQ_HANDLED;
}

static void cst3530_reset(struct cst3530 *ts)
{
	if (ts->reset) {
		gpiod_set_value_cansleep(ts->reset, 1);
		msleep(200);
		gpiod_set_value_cansleep(ts->reset, 0);
	}
	msleep(200);
}

static void cst3530_power_off(void *data)
{
	regulator_disable(data);
}

static int cst3530_probe(struct i2c_client *client)
{
	struct device *dev = &client->dev;
	struct regulator *supply;
	struct cst3530 *ts;
	struct input_dev *input;
	int ret;

	if (!i2c_check_functionality(client->adapter, I2C_FUNC_I2C))
		return -EOPNOTSUPP;
	if (client->irq <= 0)
		return dev_err_probe(dev, -EINVAL, "An interrupt is required\n");

	ts = devm_kzalloc(dev, sizeof(*ts), GFP_KERNEL);
	if (!ts)
		return -ENOMEM;
	ts->client = client;
	mutex_init(&ts->lock);
	i2c_set_clientdata(client, ts);

	supply = devm_regulator_get(dev, "vdd");
	if (IS_ERR(supply))
		return dev_err_probe(dev, PTR_ERR(supply), "Cannot get supply\n");
	ret = regulator_enable(supply);
	if (ret)
		return ret;
	ret = devm_add_action_or_reset(dev, cst3530_power_off, supply);
	if (ret)
		return ret;

	ts->reset = devm_gpiod_get_optional(dev, "reset", GPIOD_OUT_HIGH);
	if (IS_ERR(ts->reset))
		return dev_err_probe(dev, PTR_ERR(ts->reset), "Cannot get reset\n");
	cst3530_reset(ts);
	/* No documented identity register; require the command to be ACKed. */
	ret = cst3530_ack(ts);
	if (ret)
		return dev_err_probe(dev, ret, "Controller did not respond\n");

	input = devm_input_allocate_device(dev);
	if (!input)
		return -ENOMEM;
	ts->input = input;
	input->name = "Hynitron CST3530 Touchscreen";
	input->id.bustype = BUS_I2C;
	input_set_drvdata(input, ts);
	input_set_capability(input, EV_ABS, ABS_MT_POSITION_X);
	input_set_capability(input, EV_ABS, ABS_MT_POSITION_Y);
	input_set_abs_params(input, ABS_MT_PRESSURE, 0, 255, 0, 0);
	touchscreen_parse_properties(input, true, &ts->prop);
	if (!ts->prop.max_x || !ts->prop.max_y ||
	    ts->prop.max_x > 4095 || ts->prop.max_y > 4095)
		return dev_err_probe(dev, -EINVAL, "Invalid touchscreen size\n");

	ret = input_mt_init_slots(input, CST3530_MAX_CONTACTS,
				  INPUT_MT_DIRECT | INPUT_MT_DROP_UNUSED |
				  INPUT_MT_TRACK);
	if (ret)
		return ret;

	ret = devm_request_threaded_irq(dev, client->irq, NULL,
					cst3530_irq, IRQF_ONESHOT,
					dev_name(dev), ts);
	if (ret)
		return dev_err_probe(dev, ret, "Cannot request IRQ\n");

	ret = input_register_device(input);
	if (ret)
		return ret;
	dev_info(dev, "CST3530 using interrupt %d\n", client->irq);
	return 0;
}

static int __maybe_unused cst3530_suspend(struct device *dev)
{
	struct cst3530 *ts = i2c_get_clientdata(to_i2c_client(dev));

	disable_irq(ts->client->irq);
	mutex_lock(&ts->lock);
	ts->suspended = true;
	cst3530_release(ts);
	mutex_unlock(&ts->lock);
	return 0;
}

static int __maybe_unused cst3530_resume(struct device *dev)
{
	struct cst3530 *ts = i2c_get_clientdata(to_i2c_client(dev));
	int ret;

	mutex_lock(&ts->lock);
	cst3530_reset(ts);
	ret = cst3530_ack(ts);
	if (!ret)
		ts->suspended = false;
	mutex_unlock(&ts->lock);
	if (ret)
		return ret;
	enable_irq(ts->client->irq);
	return 0;
}

static SIMPLE_DEV_PM_OPS(cst3530_pm_ops, cst3530_suspend, cst3530_resume);

static const struct of_device_id cst3530_of_match[] = {
	{ .compatible = "hynitron,cst3530" },
	{ }
};
MODULE_DEVICE_TABLE(of, cst3530_of_match);

static const struct i2c_device_id cst3530_id[] = {
	{ "cst3530", 0 },
	{ }
};
MODULE_DEVICE_TABLE(i2c, cst3530_id);

static struct i2c_driver cst3530_driver = {
	.driver = {
		.name = "cst3530",
		.of_match_table = cst3530_of_match,
		.pm = &cst3530_pm_ops,
	},
	.probe_new = cst3530_probe,
	.id_table = cst3530_id,
};
module_i2c_driver(cst3530_driver);

MODULE_DESCRIPTION("Hynitron CST3530 touchscreen driver");
MODULE_LICENSE("GPL");
