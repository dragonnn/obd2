// SPDX-License-Identifier: GPL-2.0-only
/* Host test for the actual driver decoder; no kernel build is required. */
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

typedef uint8_t u8;
typedef uint16_t u16;
#include "../kernel/drivers/input/touchscreen/cst3530-report.h"

static void point(u8 *buf, unsigned int index, u16 x, u16 y, u8 pressure)
{
	u8 *p = buf + 4 + index * 5;

	p[0] = x;
	p[1] = y;
	p[2] = pressure;
	p[3] = (x >> 8) | ((y >> 8) << 4);
}

int main(void)
{
	u8 buf[CST3530_REPORT_SIZE] = { 0, 0, 0xff, 0 };
	struct cst3530_contact contacts[CST3530_MAX_CONTACTS];
	unsigned int keys, fingers;
	int n;

	assert(cst3530_decode_report(buf, contacts, 261, 927) == 0);
	buf[3] = 1;
	point(buf, 0, 261, 927, 255);
	assert(cst3530_decode_report(buf, contacts, 261, 927) == 1);
	assert(contacts[0].x == 261 && contacts[0].y == 927);
	assert(contacts[0].pressure == 255);

	point(buf, 0, 0, 0, 1);
	assert(cst3530_decode_report(buf, contacts, 261, 927) == 1);
	point(buf, 0, 262, 0, 1);
	assert(cst3530_decode_report(buf, contacts, 261, 927) == 0);
	point(buf, 0, 0, 928, 1);
	assert(cst3530_decode_report(buf, contacts, 261, 927) == 0);
	point(buf, 0, 1, 1, 0);
	assert(cst3530_decode_report(buf, contacts, 261, 927) == 0);

	/* A key record must not become a finger; invalid fingers compact out. */
	buf[3] = 0x13;
	point(buf, 0, 99, 99, 1);
	point(buf, 1, 12, 345, 67);
	point(buf, 2, 4095, 4095, 1);
	point(buf, 3, 23, 456, 89);
	assert(cst3530_decode_report(buf, contacts, 261, 927) == 2);
	assert(contacts[0].x == 12 && contacts[0].y == 345);
	assert(contacts[1].x == 23 && contacts[1].y == 456);

	/* Exercise every count combination, including the 64-byte boundary. */
	memset(buf, 0xff, sizeof(buf));
	for (keys = 0; keys < 16; keys++) {
		for (fingers = 0; fingers < 16; fingers++) {
			buf[3] = (keys << 4) | fingers;
			n = cst3530_decode_report(buf, contacts, 4095, 4095);
			if (fingers > 10 || 4 + (keys + fingers) * 5 > 64)
				assert(n == -EPROTO);
			else
				assert(n == (int)fingers);
		}
	}
	buf[3] = 0;
	buf[2] = 0;
	assert(cst3530_decode_report(buf, contacts, 261, 927) == -EPROTO);
	puts("CST3530 decoder tests passed");
	return 0;
}
