// SPDX-FileCopyrightText: 2026 Duszku <duszku511@gmail.com>
// SPDX-License-Identifier: EUPL-1.2

#include <memory.lds>

OUTPUT_ARCH(riscv64gc);
ENTRY(start);

SECTIONS
{
	. = IMAGE_ADDRESS;

	.text : {
		*(.text.start)
		*(.text)
	}
}
