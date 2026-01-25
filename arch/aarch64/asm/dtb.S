// SPDX-FileCopyrightText: 2026 Duszku <duszku511@gmail.com>
// SPDX-License-Identifier: EUPL-1.2

#include <asm/utils.S.h>

#include <section_names.h>

/// @var   fdt_blob
/// @brief Embedded flattened Devicetree blob.
FLAGS_SECTION(SNAME_DTB, SHT_PROGBITS, SHF_ALLOC)
BEGIN_OBJECT(fdt_blob)
	.incbin	BUILD_DTBO_PATH // Defined by build script.
END_OBJECT(fdt_blob)
