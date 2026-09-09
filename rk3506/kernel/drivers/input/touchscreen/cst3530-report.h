/* SPDX-License-Identifier: GPL-2.0-only */
#ifndef _CST3530_REPORT_H
#define _CST3530_REPORT_H

#include <linux/errno.h>
#include <linux/types.h>

#define CST3530_REPORT_SIZE	64
#define CST3530_MAX_CONTACTS	10
#define CST3530_HEADER_SIZE	4
#define CST3530_RECORD_SIZE	5

struct cst3530_contact {
	u16 x;
	u16 y;
	u8 pressure;
};

/* Format from the module's ESP-IDF sample; key records precede fingers. */
static inline int cst3530_decode_report(const u8 *buf,
				       struct cst3530_contact *contacts,
				       unsigned int max_x, unsigned int max_y)
{
	unsigned int fingers = buf[3] & 0x0f;
	unsigned int keys = buf[3] >> 4;
	unsigned int i, count = 0;

	if (buf[2] != 0xff || fingers > CST3530_MAX_CONTACTS ||
	    CST3530_HEADER_SIZE + (keys + fingers) * CST3530_RECORD_SIZE >
	    CST3530_REPORT_SIZE)
		return -EPROTO;

	for (i = 0; i < fingers; i++) {
		const u8 *p = buf + CST3530_HEADER_SIZE +
			      (keys + i) * CST3530_RECORD_SIZE;
		u16 x = p[0] | ((p[3] & 0x0f) << 8);
		u16 y = p[1] | ((p[3] & 0xf0) << 4);

		if (!p[2] || x > max_x || y > max_y)
			continue;

		contacts[count].x = x;
		contacts[count].y = y;
		contacts[count].pressure = p[2];
		count++;
	}

	return count;
}

#endif
