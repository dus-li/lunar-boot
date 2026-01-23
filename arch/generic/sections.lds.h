// SPDX-FileCopyrightText: 2026 Duszku <duszku511@gmail.com>
// SPDX-License-Identifier: EUPL-1.2

#pragma once

#include <section_names.h>
#include <sizes.h>

#if defined(__LINKER_SCRIPT__)

  /// Declare start text section.
  ///
  /// This is a section populated with early initialization code, implemented
  /// in assembly. Its contents are responsible for basic platform setup and
  /// creating an execution environment for HLL code.
  #define SECTION_START_TEXT              \
  	SNAME_START_TEXT : {              \
  		__text_start = .;         \
  		KEEP(*(SNAME_START_TEXT)) \
  		__etext_start = .;        \
  	}

  // Declare .data section.
  #define SECTION_DATA       \
  	.data : {            \
  		__data = .;  \
  		*(.data*)    \
  		__edata = .; \
  	}

  /// Declare .bss section.
  /// @param align Section beginning and end alignment.
  #define SECTION_BSS(align)           \
  	.bss (NOLOAD) : ALIGN(align) { \
  		__bss = .;             \
  		*(.bss*)               \
  		*(COMMON)              \
		. = ALIGN(align);      \
		__ebss = .;            \
  	}

  /// Stack space reserved for the early initialization code.
  #define SECTION_INIT_STACK(align, size) \
  	. = ALIGN(align);                 \
  	. += size;                        \
  	__estack = .;                     \

#endif // defined(__LINKER_SCRIPT__)
