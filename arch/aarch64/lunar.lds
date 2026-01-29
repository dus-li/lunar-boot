// SPDX-FileCopyrightText: 2026 Duszku <duszku511@gmail.com>
// SPDX-License-Identifier: EUPL-1.2

#include <memory.lds.h>
#include <sections.lds.h>

OUTPUT_ARCH(aarch64);
ENTRY(start);

PAGE_SIZE = _4_KiB;

SECTIONS
{
	// Start placing sections at board-defined firmware address.
	. = IMAGE_ADDRESS;

	// Devicetree blob.
	SECTION_DTB

	// Unreclaimable code.
	SECTION_TEXT

	// Initialized and uninitialized data sections.
	SECTION_DATA
	SECTION_BSS(16)

	// Stack space for initialization code.
	SECTION_INIT_STACK(16, _4_KiB)

	// Early initialization sections, in reverse reclaim order.
	SECTION_START_ARENA(PAGE_SIZE, PAGE_SIZE)
	SECTION_START_TEXT(PAGE_SIZE)

	// Dynamic allocations for late code.
	SECTION_HEAP(PAGE_SIZE)
}
