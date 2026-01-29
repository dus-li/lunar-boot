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
  /// creating an execution environment for HLL code. The contents of this
  /// section will be reclaimed once the setup is complete.
  #define SECTION_START_TEXT(align)       \
  	SNAME_START_TEXT : ALIGN(align) { \
  		__start = .;              \
  		KEEP(*(SNAME_START_TEXT)) \
  		__estart = .;             \
  	}

  /// Declare standard text section.
  ///
  /// This is a section with executable code which is not a part of the early
  /// initialization. As such, it will not be reclaimed after the initial
  /// setup is complete.
  #define SECTION_TEXT       \
  	.text : {            \
  		__text = .;  \
  		*(.text*)    \
  		__etext = .; \
  	}

  /// Declare a section with an embedded Devicetree blob.
  #define SECTION_DTB              \
  	SNAME_DTB : ALIGN(8) {     \
  		__dtb = .;         \
  		KEEP(*(SNAME_DTB)) \
  		__edtb = .;        \
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

  /// Heap space.
  #define SECTION_HEAP(align) \
  	. = ALIGN(align);     \
  	__heap = .;

#endif // defined(__LINKER_SCRIPT__)
