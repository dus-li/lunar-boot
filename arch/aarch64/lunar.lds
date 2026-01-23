// SPDX-FileCopyrightText: 2026 Duszku <duszku511@gmail.com>
// SPDX-License-Identifier: EUPL-1.2

#include <memory.lds.h>
#include <sections.lds.h>

OUTPUT_ARCH(aarch64);
ENTRY(start);

SECTIONS
{
	// Start placing sections at board-defined firmware address.
	. = IMAGE_ADDRESS;

	// Early initialization code.
	SECTION_START_TEXT

	// Initialized and uninitialized data sections.
	SECTION_DATA
	SECTION_BSS(16)

	// Stack space for initialization code.
	SECTION_INIT_STACK(16, _4_KiB)
}
